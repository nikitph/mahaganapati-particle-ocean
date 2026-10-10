pub const W: usize = 1920;
pub const H: usize = 1080;
pub const FPS: usize = 30;
pub const FRAMES: usize = 360;
pub const NEAR: f32 = 650.0;
pub const FAR: f32 = 5000.0;
pub const BINS: usize = 32;
pub fn smooth(a:f32,b:f32,v:f32)->f32 {
    let t=((v-a)/(b-a)).clamp(0.0,1.0); t*t*(3.0-2.0*t)
}
pub fn rnd(seed:u32,i:u32)->f32 {
    let mut x=seed.wrapping_add(i.wrapping_mul(0x9e3779b9));
    x^=x>>16; x=x.wrapping_mul(0x7feb352d); x^=x>>15;
    x=x.wrapping_mul(0x846ca68b); x^=x>>16; x as f32/u32::MAX as f32
}
#[derive(Clone,Copy)]
pub struct V {pub x:f32,pub y:f32,pub z:f32}
impl V {
    pub fn new(x:f32,y:f32,z:f32)->Self {Self{x,y,z}}
    pub fn add(self,b:Self)->Self {Self::new(self.x+b.x,self.y+b.y,self.z+b.z)}
    pub fn sub(self,b:Self)->Self {Self::new(self.x-b.x,self.y-b.y,self.z-b.z)}
    pub fn mul(self,k:f32)->Self {Self::new(self.x*k,self.y*k,self.z*k)}
    pub fn dot(self,b:Self)->f32 {self.x*b.x+self.y*b.y+self.z*b.z}
    pub fn cross(self,b:Self)->Self {Self::new(self.y*b.z-self.z*b.y,self.z*b.x-self.x*b.z,self.x*b.y-self.y*b.x)}
    pub fn unit(self)->Self {self.mul(1.0/self.dot(self).sqrt())}
}
pub struct Camera {pub pos:V, pub right:V,pub up:V,pub forward:V,pub focal:f32}
impl Camera {
    pub fn at(t:f32)->Self {
        let rise=smooth(0.0,7.8,t);
        let pos=V::new(-260.0+300.0*rise,-85.0+170.0*rise,-1800.0+65.0*smooth(8.0,12.0,t));
        let aim=V::new(0.0,-45.0+55.0*rise,100.0);
        let forward=aim.sub(pos).unit();
        let right=forward.cross(V::new(0.0,1.0,0.0)).mul(-1.0).unit();
        let up=forward.cross(right).unit();
        Self{pos,right,up,forward,focal:1830.0}
    }
    pub fn project(&self,p:V)->(f32,f32,f32,f32) {
        let rel=p.sub(self.pos); let d=rel.dot(self.forward);
        let scale=self.focal/d.max(100.0);
        (960.0+rel.dot(self.right)*scale,540.0-rel.dot(self.up)*scale,d,scale)
    }
}
pub fn focus(t:f32)->f32 {1050.0+850.0*smooth(0.5,4.5,t)}
