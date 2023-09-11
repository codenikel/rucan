use radiant_core::{RadiantMessage, RadiantTool};
use radiant_main::{RadiantApp, RadiantResponse};
use winit::{event_loop::EventLoop, window::WindowBuilder};

async fn run() {
    let env = env_logger::Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let handler: Box<dyn Fn(RadiantResponse)> = Box::new(move |response: RadiantResponse| {
        println!("Response: {:?}", response);
    });

    let mut app = RadiantApp::new(window, handler).await;
    app.handle_message(RadiantMessage::SelectTool(RadiantTool::Rectangle));

    event_loop.run(move |event, _, control_flow| {
        if let Some(response) = app.handle_event(event, control_flow) {
            println!("Response: {:?}", response);
        }
    });
}

fn main() {
    pollster::block_on(run());
}
