"""Original synthesized stereo soundscape: airborne grains, low drone, arrival chime."""
from pathlib import Path
import numpy as np
from scipy.signal import butter, sosfilt
from scipy.io.wavfile import write

root=Path(__file__).resolve().parent
sr=48000
t=np.arange(sr*12)/sr
rng=np.random.default_rng(3087)
audio=np.zeros((len(t),2))

def add(signal, pan=0.0):
    angle=(np.asarray(pan)+1)*np.pi/4
    audio[:,0]+=signal*np.cos(angle)
    audio[:,1]+=signal*np.sin(angle)

fade=np.minimum(t/1.8,1)*np.minimum((12-t)/0.65,1)
pad_env=fade*(0.025+0.025*np.exp(-((t-7.2)/2.6)**2))
for n,f in enumerate([73.416,110.0,146.832,220.0,293.664]):
    tone=np.sin(2*np.pi*f*t+0.35*np.sin(t*0.22+n))
    tone+=0.18*np.sin(2*np.pi*f*2.002*t)
    add(tone*pad_env/(1+n*0.6),0.42*np.sin(n*2.0))

for channel in range(2):
    noise=rng.normal(size=len(t))
    wind=sosfilt(butter(3,[180,2200],btype='bandpass',fs=sr,output='sos'),noise)
    wind/=np.std(wind)
    env=(0.003+0.012*np.exp(-((t-4.3)/1.8)**2)+0.012*np.exp(-((t-6.8)/0.9)**2))*fade
    audio[:,channel]+=wind*env

# Small grain swarms move from the edges toward the center.
for start,pan in [(2.6,-0.65),(4.1,0.65),(5.5,-0.35),(6.6,0.3)]:
    env=np.exp(-((t-start)/0.38)**2)
    carrier=np.sin(2*np.pi*(530*t+40*np.sin(t*1.8)))
    add(carrier*env*0.004,pan)

for k,f in enumerate([587.328,880.0,1174.656]):
    age=np.maximum(t-(7.45+k*0.13),0)
    env=(1-np.exp(-age*65))*np.exp(-age*(1.0+k*0.18))*(t>=7.45+k*0.13)
    tone=np.sin(2*np.pi*f*age)+0.20*np.sin(2*np.pi*f*2.76*age)*np.exp(-age*2.0)
    add(tone*env*0.027/(1+k*0.4),(-0.35+k*0.35))

audio*=np.minimum((12-t)/0.6,1)[:,None]
peak=np.max(np.abs(audio))
audio=audio/peak*0.78
(root/'passes').mkdir(exist_ok=True)
write(root/'passes'/'score-raw.wav',sr,audio.astype(np.float32))
print('synthesized',len(t)/sr,'seconds; peak',float(np.max(np.abs(audio))))
