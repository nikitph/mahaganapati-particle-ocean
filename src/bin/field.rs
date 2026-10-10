use fframes::{AudioMap,Color,Duration,EncoderOptions,FFramesContext,Frame,RenderOptions,Svgr,Video,cli};
use fframes_skia_renderer::{SkiaFFramesRenderer,SkiaPipelineConfig,metal::SkiaMetalCtx};
use ganesha_volume::*;
use std::{fmt::Write,process::ExitCode};

const COLORS:usize=58;
struct Target{p:V,start:V,delay:f32,palette:usize,r:f32}
struct Field{target:Vec<Target>,palette:Vec<[u8;3]>}
fn wave(x:f32,z:f32,t:f32)->f32 {
    -450.0+27.0*(x*0.004+z*0.003-t*0.8).sin()
        +16.0*(x*0.009-z*0.005+t*1.1).sin()
        +7.0*(x*0.021+z*0.012-t*1.7).sin()
}
// A single rising swell deforms the same material coordinates for both sea and figure.
fn surface(x:f32,z:f32,t:f32)->V {
    let crest=1250.0-1330.0*smooth(0.3,4.9,t);
    let growth=smooth(0.6,4.3,t)*(1.0-smooth(5.1,8.4,t));
    let ridge=(-((z-crest)/480.0).powi(2)).exp();
    let span=(-(x/1120.0).powi(4)).exp();
    let lift=840.0*growth*ridge*span;
    let curl=210.0*growth*span*ridge;
    V::new(x+22.0*growth*(z*0.009+t).sin()*ridge,
        wave(x,z,t)+lift,z-curl)
}
impl Field {
    fn new()->Self {
        let bytes=include_bytes!("../target.bin");
        let n=u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let mut palette:Vec<[u8;3]>=(0..48).map(|i|[bytes[4+i*3],bytes[5+i*3],bytes[6+i*3]]).collect();
        for c in &mut palette {
            let v=(0.2126*c[0] as f32+0.7152*c[1] as f32+0.0722*c[2] as f32) as u8;
            *c=[v,v,v];
        }
        palette.extend([60,85,110,135,160,187,215,239,252,190].map(|v|[v,v,v]));
        let target=(0..n).map(|i| {
            let k=i as u32;let off=148+i*5;
            let x=u16::from_le_bytes(bytes[off..off+2].try_into().unwrap()) as f32;
            let y=u16::from_le_bytes(bytes[off+2..off+4].try_into().unwrap()) as f32;
            let col=bytes[off+4] as usize;
            let lum=(palette[col][0] as f32+palette[col][1] as f32+palette[col][2] as f32)/765.0;
            let z=100.0-80.0*(x/532.0*3.14159).sin()*(y/560.0*3.14159).sin()-25.0*(1.0-lum);
            let sx=(x-266.0)*2.7+(rnd(11,k)-0.5)*90.0;
            let sz=-100.0+1050.0*(y/560.0).powf(0.8)+(rnd(13,k)-0.5)*55.0;
            Target{p:V::new((x-266.0)*1.62+(rnd(29,k)-0.5)*1.5,(280.0-y)*1.62,z),
                start:V::new(sx,wave(sx,sz,0.0),sz),
                delay:4.0+0.30*(y/560.0)+rnd(19,k)*0.15,palette:col,r:0.82+0.65*rnd(23,k)}
        }).collect();
        Self{target,palette}
    }
    fn position(p:&Target,t:f32)->V {
        let b=smooth(p.delay,p.delay+3.7,t);
        let sea=surface(p.start.x,p.start.z,t);
        // The crest retains its grains: the wave sheet contracts into the sampled relief.
        // There are no emitters or incoming particle streams.
        sea.mul(1.0-b).add(p.p.mul(b))
    }
}

fn disc(path:&mut String,x:f32,y:f32,r:f32) {
    let r=r.clamp(0.35,9.0);
    let _=write!(path,"M{:.1} {:.1}a{:.1} {:.1} 0 1 0 {:.1} 0a{:.1} {:.1} 0 1 0 -{:.1} 0",x-r,y,r,r,2.0*r,r,r,2.0*r);
}
fn add(paths:&mut [String],depths:&mut [String],cam:&Camera,p:V,r:f32,col:usize) {
    let (x,y,d,s)=cam.project(p);
    if d<NEAR || x< -20.0 || x>W as f32+20.0 || y< -20.0 || y>H as f32+20.0 {return;}
    let bin=(((d-NEAR)/(FAR-NEAR)*(BINS-1) as f32) as usize).min(BINS-1);
    disc(&mut paths[bin*COLORS+col],x,y,r*s);
    disc(&mut depths[bin],x,y,r*s);
}
impl Video for Field {
    const FPS:usize=FPS; const WIDTH:usize=W; const HEIGHT:usize=H*2;
    const BACKGROUND_COLOR:Color=Color::WHITE;
    fn duration(&self)->Duration<'_>{Duration::Frames(FRAMES)}
    fn audio(&self)->AudioMap<'_>{AudioMap::none()}
    fn render_frame<'a>(&'a self,frame:Frame,_ctx:&FFramesContext<'a,'_>)->Svgr<'a>{
        let t=frame.index as f32/FPS as f32;let cam=Camera::at(t);
        let mut paths:Vec<String>=(0..BINS*COLORS).map(|_|String::new()).collect();
        let mut depths:Vec<String>=(0..BINS).map(|_|String::new()).collect();
        for i in 0..135_000u32 {
            let bx=(rnd(101,i)*2.0-1.0)*4200.0;
            let bz=-1000.0+7800.0*rnd(103,i).powf(1.25);
            let x=bx+95.0*(bz*0.0018+t*0.34).sin();
            let z=bz+65.0*(bx*0.002-t*0.48).sin();
            let surface_point=surface(x,z,t);
            let y=surface_point.y;
            let slope=(surface(x+10.0,z,t).y-y)/10.0;
            let shade=((3.5+2.0*slope+2.4*(rnd(107,i)-0.5)) as usize).min(7);
            let r=0.8+1.25*rnd(109,i);
            add(&mut paths,&mut depths,&cam,surface_point,r,48+shade);
            if i%107==0 && (t*1.8+bx*0.003+bz*0.002).sin()>0.55 {
                add(&mut paths,&mut depths,&cam,V::new(x,y+1.0,z),r*1.6,56);
            }
        }
        for (i,p) in self.target.iter().enumerate() {
            let pos=Self::position(p,t);
            let settle=smooth(p.delay,p.delay+3.7,t);
            
            let shade=(2.0+4.0*rnd(107,i as u32)) as usize;
            let col=if settle<0.66+0.30*rnd(307,i as u32){48+shade}else{p.palette};
            // Short exposure samples provide motion streaks following the real trajectory.
            let previous=Self::position(p,(t-0.012).max(0.0));
            let (_,_,_,s)=cam.project(pos);
            if pos.sub(previous).dot(pos.sub(previous))*s*s>9.0 {
                add(&mut paths,&mut depths,&cam,previous,p.r*0.65,col);
            }
            add(&mut paths,&mut depths,&cam,pos,p.r,col);
        }
        // A small number of foreground airborne motes reveal the lens's depth of field.
        for i in 0..220u32 {
            let p=V::new((rnd(211,i)-0.5)*2200.0+35.0*(t*0.3+rnd(213,i)*6.0).sin(),
                -400.0+900.0*rnd(217,i)+t*4.0,-1000.0+650.0*rnd(223,i));
            add(&mut paths,&mut depths,&cam,p,0.6+1.5*rnd(227,i),57);
        }
        let mut color_nodes:Vec<Svgr>=Vec::new();let mut depth_nodes:Vec<Svgr>=Vec::new();
        for bin in (0..BINS).rev(){
            let fog=(bin as f32/(BINS-1) as f32).powf(1.6)*0.84;
            for col in 0..COLORS {
                let d=std::mem::take(&mut paths[bin*COLORS+col]);if d.is_empty(){continue;}
                let rgb=self.palette[col];let sky=[244.0,244.0,244.0];
                let c:Vec<u8>=(0..3).map(|k|(rgb[k] as f32*(1.0-fog)+sky[k]*fog) as u8).collect();
                let fill=format!("#{:02x}{:02x}{:02x}",c[0],c[1],c[2]);
                color_nodes.push(fframes::svgr!(<path d={d} fill={fill} />));
            }
            let d=std::mem::take(&mut depths[bin]);let gray=(bin as f32/(BINS-1) as f32*255.0) as u8;
            let fill=format!("#{gray:02x}{gray:02x}{gray:02x}");
            depth_nodes.push(fframes::svgr!(<path d={d} fill={fill} />));
        }
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width={W} height={H*2}>
            <defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#fafafa" /><stop offset="70%" stop-color="#ededed" />
                <stop offset="100%" stop-color="#d4d4d4" /></linearGradient></defs>
            <rect width={W} height={H} fill="url(#sky)" />
            {color_nodes}
            <g transform={format!("translate(0 {H})")}>
                <rect width={W} height={H} fill="#ffffff" />{depth_nodes}
            </g>
        </svg>)
    }
}
fn main()->ExitCode {
    let v=Field::new();let gpu=SkiaMetalCtx::new(W,H*2).unwrap();
    cli::new(&v,RenderOptions{video_encoder_options:EncoderOptions{
        preferred_encoder:Some("libx264"),codec_params:Some(&[("crf","10"),("preset","fast")]),..Default::default()},..Default::default()})
    .backend(SkiaFFramesRenderer::new_metal(&gpu,SkiaPipelineConfig::default()).unwrap())
    .default_output("passes/field.mp4").run()
}
