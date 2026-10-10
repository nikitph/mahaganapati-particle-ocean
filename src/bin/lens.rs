use fframes::{AudioMap,AudioTimestamp::*,AudioTrack,Color,Duration,EncoderOptions,FFramesContext,FFramesSyncedVideoFrame,Frame,MediaDirectory,RenderOptions,Shader,ShaderUniforms,Svgr,SyncVideoFrameInput,Video,cli};
use fframes_skia_renderer::{SkiaFFramesRenderer,SkiaPipelineConfig,metal::SkiaMetalCtx};
use ganesha_volume::*;
use std::process::ExitCode;
struct Lens{shader:Shader}
impl Video for Lens {
    const FPS:usize=FPS;const WIDTH:usize=W;const HEIGHT:usize=H;
    const BACKGROUND_COLOR:Color=Color::WHITE;
    fn duration(&self)->Duration<'_>{Duration::Frames(FRAMES)}
    fn audio(&self)->AudioMap<'_>{AudioMap::from([AudioTrack::new("score.wav",Second(0.0)..Eof)])}
    fn render_frame<'a>(&'a self,frame:Frame,ctx:&FFramesContext<'a,'_>)->Svgr<'a>{
        let input=SyncVideoFrameInput{start_from:0.0,looping:false,editor_fallback_image:None};
        let Some(atlas)=frame.get_synced_video_frame(ctx,"field.mp4",&input) else {
            return fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={W} height={H}><rect width={W} height={H} fill="#faf8ef" /></svg>);
        };
        let atlas=atlas.into_image();let t=frame.index as f32/FPS as f32;
        let uniforms=ShaderUniforms::new().image("iChannel0",&atlas).float("uFocus",focus(t))
            .float("uAperture",22.0-6.0*smooth(6.0,8.5,t)).float("uReveal",smooth(5.5,8.3,t));
        let layer=self.shader.draw(&frame,uniforms);
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={W} height={H}>
            <image href={layer.href()} x="0" y="0" width={W} height={H} />
        </svg>)
    }
}
fn main()->ExitCode{
    let directory=MediaDirectory::read_folder("passes").unwrap();let media=directory.process_media_source().unwrap();
    let v=Lens{shader:Shader::sksl(include_str!("../lens.sksl"))};let gpu=SkiaMetalCtx::new(W,H).unwrap();
    cli::new(&v,RenderOptions{media:Some(&media),video_encoder_options:EncoderOptions{
        preferred_encoder:Some("libx264"),codec_params:Some(&[("crf","17"),("preset","slow"),("tune","grain")]),..Default::default()},..Default::default()})
    .backend(SkiaFFramesRenderer::new_metal(&gpu,SkiaPipelineConfig::default()).unwrap())
    .default_output("mahaganapati-wave.mp4").run()
}
