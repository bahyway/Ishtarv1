/* ============================================================================
   BahyWay Šala House Standard — NAṢĀRU INSTRUMENTS (ŠALA-STD-001, N2–N4)
   Camera + selection machinery every court imports by default.
   Usage:
     var cam=NasaruCamera({yaw:0.6,pitch:0.35,z:1});
     cam.bindPointer(stageEl,{onClick:function(mx,my){...}});   // N2 drag/zoom/guard
     cam.project(x,y,z,cv)                    -> {x,y,f}
     cam.landOn({yaw,pitch,z})                // N3: depart to space, descend on target
     cam.toggleBird()                         // N4: gliding flight above the membrane
     cam.tick(nowMs)                          // each frame before projecting
     NasaruBounce(selT,nowMs)                 -> vertical offset 0..1 (decaying |sin|)
   Any pointer input interrupts bird-flight and landing, returning control.
   ========================================================================== */
function NasaruCamera(init){
  var cam={yaw:init.yaw||0.6,pitch:init.pitch||0.35,z:init.z||1,
    px:init.px||0,py:init.py||0,cx:0,cy:0,cz:0};   /* cx,cy,cz: look-at center (N9) */
  var landing=null,bird=false,birdT=0,drag=null,dragMoved=false,dragDist=0;
  function tick(now){
    if(landing){
      var ph=landing.t<0.35?'up':'down';
      landing.t=Math.min(1,landing.t+0.016);
      var e=landing.t<0.35?landing.t/0.35:(landing.t-0.35)/0.65;
      e=e*e*(3-2*e);
      if(landing.t<0.35){ /* depart to space */
        cam.z=landing.z0+(landing.zSpace-landing.z0)*e;
        cam.yaw=landing.yaw0+(landing.tgt.yaw-landing.yaw0)*e*0.4;
        cam.pitch=landing.p0+(0.9-landing.p0)*e*0.5;
      }else{ /* descend onto the address */
        cam.z=landing.zSpace+(landing.tgt.z-landing.zSpace)*e;
        cam.yaw=cam.yaw+(landing.tgt.yaw-cam.yaw)*0.10;
        cam.pitch=cam.pitch+(landing.tgt.pitch-cam.pitch)*0.10;
      }
      if(landing.t>=1){landing=null;}
    }else if(bird){
      birdT+=0.016;
      cam.yaw+=0.0035;                                  /* slow circuit */
      cam.pitch=0.72+0.16*Math.sin(birdT*0.5);          /* wingbeat breathing */
      cam.z=cam.z+( (1.35+0.25*Math.sin(birdT*0.23)) - cam.z )*0.02;
    }
  }
  function interrupt(){landing=null;bird=false;}
  function landOn(tgt){
    interrupt();
    landing={tgt:tgt,t:0,z0:cam.z,yaw0:cam.yaw,p0:cam.pitch,
      zSpace:Math.max(0.42,Math.min(0.6,cam.z*0.4))};}
  function toggleBird(){bird=!bird;if(bird)landing=null;return bird;}
  function project(x,y,z,cv){
    x-=cam.cx; y-=cam.cy; z-=cam.cz;
    var cy=Math.cos(cam.yaw),sy=Math.sin(cam.yaw);
    var cp=Math.cos(cam.pitch),sp=Math.sin(cam.pitch);
    var X=x*cy+z*sy,Z=-x*sy+z*cy;
    var Y=y*cp-Z*sp,Z2=y*sp+Z*cp;
    var S=Math.min(cv.width,cv.height)*0.5*cam.z;
    var f=2.6/(2.6+Z2);
    return {x:cv.width*0.42+X*S*f+cam.px,y:cv.height*0.50-Y*S*f+cam.py,f:f};}
  function bindPointer(stage,opts){
    opts=opts||{};
    stage.addEventListener('wheel',function(e){e.preventDefault();interrupt();
      cam.z=Math.min(8,Math.max(0.4,cam.z*(e.deltaY<0?1.12:0.89)));
      if(opts.onWheel)opts.onWheel();},{passive:false});
    stage.addEventListener('mousedown',function(e){if(e.button!==0)return;
      interrupt();drag={x:e.clientX,y:e.clientY};dragMoved=false;dragDist=0;});
    window.addEventListener('mouseup',function(){drag=null;});
    stage.addEventListener('mousemove',function(e){
      if(!drag)return;
      var dx=e.clientX-drag.x,dy=e.clientY-drag.y;
      dragDist+=Math.abs(dx)+Math.abs(dy);if(dragDist>9)dragMoved=true;
      cam.yaw+=dx*0.005;
      cam.pitch=Math.max(-0.9,Math.min(1.35,cam.pitch+dy*0.005));
      drag={x:e.clientX,y:e.clientY};});
    stage.addEventListener('click',function(e){
      if(dragMoved)return;
      var r=stage.getBoundingClientRect();
      if(opts.onClick)opts.onClick(e.clientX-r.left,e.clientY-r.top);});
  }
  return {cam:cam,tick:tick,project:project,landOn:landOn,
    toggleBird:toggleBird,interrupt:interrupt,bindPointer:bindPointer,
    get landing(){return !!landing;},get bird(){return bird;},
    get dragMoved(){return dragMoved;}};
}
/* ---- N9 · CINEMATIC CAMERA (SALA-STD-001-A1) --------------------------------
   Look-at tracking in the membrane. Modes:
     'off'        — manual control
     'wavefront'  — dolly with a propagating front (target = front position)
     'follow'     — damped chase of a particle/structure centroid
     'crane'      — hold target, slow rise-and-fall sweep
     'orbit'      — circle the target while tracking it
   Usage: var CIN=NasaruCinema(NAV);
          CIN.setMode('wavefront'); CIN.cycle();
          CIN.tick(now, target);               // target={x,y,z}; call AFTER NAV.tick
   Any user drag/wheel should set CIN.setMode('off') — the human outranks
   the director. Landing-from-space outranks the cinema.
   MEMBRANE MOTION DOCTRINE: dynamics are rendered as deformation of the
   membrane surface, particles RIDING it — never dots in empty space. */
function NasaruCinema(NAVOBJ){
  var mode='off',t0=0,MODES=['off','wavefront','follow','crane','orbit'];
  function setMode(m){mode=m;t0=0;}
  function cycle(){var i=(MODES.indexOf(mode)+1)%MODES.length;mode=MODES[i];t0=0;return mode;}
  function tick(now,target){
    if(mode==='off'||!target||NAVOBJ.landing)return;
    var cam=NAVOBJ.cam; t0+=0.016;
    var k=0.08;
    cam.cx+=(target.x-cam.cx)*k;
    cam.cy+=((target.y||0)-cam.cy)*k;
    cam.cz+=((target.z||0)-cam.cz)*k;
    if(mode==='wavefront'){
      cam.pitch+=(0.44-cam.pitch)*0.04;
      cam.z+=(2.2-cam.z)*0.03;
      cam.yaw+=Math.sin(t0*0.35)*0.0009;            /* gentle handheld drift */
    }else if(mode==='follow'){
      cam.pitch+=(0.55-cam.pitch)*0.04;
      cam.z+=(2.6-cam.z)*0.03;
    }else if(mode==='crane'){
      cam.pitch+=((0.95+0.30*Math.sin(t0*0.4))-cam.pitch)*0.05;
      cam.z+=((1.6+0.35*Math.sin(t0*0.27))-cam.z)*0.03;
    }else if(mode==='orbit'){
      cam.yaw+=0.005;
      cam.pitch+=(0.5-cam.pitch)*0.03;
      cam.z+=(2.0-cam.z)*0.03;
    }
  }
  return {get mode(){return mode;},setMode:setMode,cycle:cycle,tick:tick,MODES:MODES};
}
function NasaruBounce(selT,now){
  var bt=(now-selT)/1000;
  return Math.abs(Math.sin(bt*6))*Math.exp(-bt*0.3);}
if(typeof module!=='undefined')module.exports={NasaruCamera:NasaruCamera,NasaruBounce:NasaruBounce,NasaruCinema:NasaruCinema};
