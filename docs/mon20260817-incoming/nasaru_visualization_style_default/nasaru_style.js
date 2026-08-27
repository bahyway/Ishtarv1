"use strict";
/* ═══ NASARU DEFAULT VISUALIZING STYLE — the shared interaction core ═══
   GL-VIZ-001. The single canonical implementation of the four interactions
   every Šala prototype must have:
     (1) scroll-zoom (sky↔ground)
     (2) 3D moving membrane (drag rotate/tilt + optional ṬĀLUKU auto-orbit)
     (3) left-click bounce-in-place → StoryEngine journal (born only)
     (4) right-click context menu branching by birth-status
   Plus optional land-from-sky. Production is egui/WGPU; this is the Šala
   prototype grammar. Attach: window.NasaruStyle.create(opts) -> controller.

   Particle shape expected by the core:
     { id, pos:[x,y,z], born:bool, kaki?:string,
       story?:[{t,e}], refusalReason?:string, bounce:0 (managed) }
*/
(function(global){
  function create(opts){
    const cv       = opts.canvas;
    const ctx      = cv.getContext("2d");
    const particles= opts.particles || [];
    const onDraw   = opts.onDraw   || function(){};      // (ctx, api) => draw membrane/extra BEFORE particles
    const onSelect = opts.onSelect || function(){};      // (particle) => void, after bounce+story
    const storyEl  = opts.storyEl  || null;              // element to render StoryEngine journal
    const menuEl   = opts.menuEl   || null;              // element for the right-click context menu
    const logFn    = opts.logFn    || function(){};
    const REDUCED  = opts.reducedMotion != null ? opts.reducedMotion
                     : (global.matchMedia && global.matchMedia("(prefers-reduced-motion: reduce)").matches);
    const ZMIN=opts.zoomMin!=null?opts.zoomMin:0.55, ZMAX=opts.zoomMax!=null?opts.zoomMax:4.0;

    const cam = { yaw:0.6, pitch:0.35, k:1.2 };
    let orbit = opts.autoOrbit!=null ? opts.autoOrbit : !REDUCED;
    let selected = null, land = null, tg = 0;

    /* ── 3D projection (shared) ── */
    function project(w){
      const cy=Math.cos(cam.yaw), sy=Math.sin(cam.yaw);
      let x=w[0]*cy+w[2]*sy, z=-w[0]*sy+w[2]*cy, y=w[1];
      const cp=Math.cos(cam.pitch), sp=Math.sin(cam.pitch);
      const y2=y*cp - z*sp, z2=y*sp + z*cp;
      const d=520/(520+z2*1.3*cam.k);
      return { x:cv.width/2 + x*cam.k*d, y:cv.height/2 + y2*cam.k*d, d, z:z2 };
    }
    const api = { project, cam, get selected(){return selected;} };

    /* ── draw ── */
    function draw(){
      ctx.setTransform(1,0,0,1,0,0);
      ctx.fillStyle = opts.bg || "#0d1119";
      ctx.fillRect(0,0,cv.width,cv.height);
      onDraw(ctx, api);                         // membrane / caller extras first
      /* particles, depth-sorted, with bounce lift + stem for the selected */
      const items = particles.map(p=>{
        const lift = [p.pos[0], p.pos[1]-p.bounce*46, p.pos[2]];
        return { p, base:project(p.pos), pr:project(lift) };
      }).sort((a,b)=>b.pr.z - a.pr.z);
      for(const it of items){
        const p=it.p, sel=(selected===p.id);
        const col = p.born
          ? (sel ? "#e8e2d4" : (p.color || "#d9a441"))
          : "rgba(212,85,63,.82)";
        if(p.born){
          ctx.fillStyle=col;
          ctx.beginPath(); ctx.arc(it.pr.x, it.pr.y, (sel?4.6:2.6)*it.pr.d, 0, 7); ctx.fill();
          if(sel){
            ctx.strokeStyle="#d9a441";
            ctx.beginPath(); ctx.arc(it.pr.x, it.pr.y, 9*it.pr.d, 0, 7); ctx.stroke();
            if(p.bounce>0.02){ ctx.strokeStyle="rgba(217,164,65,.35)";
              ctx.beginPath(); ctx.moveTo(it.base.x,it.base.y); ctx.lineTo(it.pr.x,it.pr.y); ctx.stroke(); }
          }
        } else {
          ctx.fillStyle=col;
          ctx.fillRect(it.pr.x-1.7, it.pr.y-1.7, 3.4, 3.4);  // refused = square, no lift
        }
        it.p._sx=it.pr.x; it.p._sy=it.pr.y;                  // cache for hit-test
        if(p.bounce>0.01) p.bounce*=0.9;
      }
      /* watermark (Šala rehearsal mark) */
      if(opts.watermark!==false){
        ctx.save(); ctx.translate(cv.width/2, 30); ctx.rotate(-0.06);
        ctx.fillStyle="rgba(217,164,65,0.07)"; ctx.font="bold 26px monospace";
        ctx.textAlign="center"; ctx.fillText("S I M U L A T I O N", 0, 0);
        ctx.restore(); ctx.textAlign="left";
      }
    }
    api.draw = draw;

    /* ── hit-test ── */
    function hit(mx,my){
      let best=null, bd=13;
      for(const p of particles){ if(p._sx==null) continue;
        const d=Math.hypot(p._sx-mx, p._sy-my);
        if(d<bd){ bd=d; best=p; } }
      return best;
    }

    /* ── (3) left-click bounce-in-place → StoryEngine (born only) ── */
    function leftClick(mx,my){
      const p=hit(mx,my);
      if(!p) return;
      if(!p.born){                                          // refused: no story on left-click
        logFn("● refused Non-Particle — no StoryEngine (never born); right-click for its Refusal Record","bad");
        return;
      }
      selected=p.id; p.bounce=0.5;
      renderStory(p);
      onSelect(p);
      logFn("● Particle #"+p.id+" bounces in place · StoryEngine opened (left-click)","ok");
      draw();
    }
    function renderStory(p){
      if(!storyEl) return;
      storyEl.innerHTML =
        '<div class="ns-sh">Particle <b>#'+p.id+'</b>'+(p.kaki?' · KAKI '+p.kaki+'…':'')+'</div>'+
        '<div class="ns-stl">StoryEngine journal (GL-STY-001):</div>'+
        (p.story||[]).map(function(e){
          return '<div class="ns-sev"><span class="ns-st">t+'+e.t+'</span> '+e.e+'</div>';
        }).join("") +
        '<div class="ns-innoc">this Particle is an individual — non-substitutable (GL-ONT-002)</div>';
    }
    api.renderStory = renderStory;

    /* ── (4) right-click context menu, branching by birth-status ── */
    function rightClick(e, mx, my){
      if(!menuEl) return;
      const p=hit(mx,my);
      if(!p){ menuEl.style.display="none"; return; }
      if(p.born){
        menuEl.innerHTML =
          '<div class="ns-mh">Inner Particle #'+p.id+(p.kaki?' · KAKI '+p.kaki+'…':'')+'</div>'+
          '<div class="ns-mi" data-a="story">📖 Open StoryEngine journal</div>'+
          '<div class="ns-mi" data-a="cohort">◎ Add to cohort</div>'+
          '<div class="ns-mnote">born · individual · queryable by identity</div>';
      } else {
        menuEl.innerHTML =
          '<div class="ns-mh" style="color:#d4553f">Refused Non-Particle</div>'+
          '<div class="ns-mi" data-a="refusal">⛔ Show Refusal Record</div>'+
          '<div class="ns-mnote">no KAKI · no StoryEngine (never born) · Bāb Ṭurdi only</div>';
      }
      menuEl.style.display="block";
      menuEl.style.left = Math.min(e.clientX, global.innerWidth-230) + "px";
      menuEl.style.top  = e.clientY + "px";
      menuEl.querySelectorAll(".ns-mi").forEach(function(mi){
        mi.onclick=function(){
          const a=mi.dataset.a; menuEl.style.display="none";
          if(a==="story"){ selected=p.id; p.bounce=0.5; renderStory(p); onSelect(p);
            logFn("StoryEngine opened for Particle #"+p.id+" (right-click)","ok"); draw(); }
          else if(a==="cohort"){ logFn("＋ Particle #"+p.id+" added to cohort (identity-safe)","ok"); }
          else if(a==="refusal"){
            if(storyEl) storyEl.innerHTML =
              '<div class="ns-sh" style="color:#d4553f">Refusal Record · Bāb Ṭurdi (no identity)</div>'+
              '<div>reason: '+(p.refusalReason||"REFUSED")+'</div>'+
              '<div class="ns-innoc" style="color:#8b93a3">no StoryEngine: never born, no KAKI — contributes only to its client\'s cost signature</div>';
            logFn("Refusal Record shown (right-click) — no story, because never born","bad");
          }
        };
      });
    }

    /* ── (1)(2) camera: drag rotate/tilt, wheel zoom, click discrimination ── */
    function px(e){ const r=cv.getBoundingClientRect();
      return { x:(e.clientX-r.left)*cv.width/r.width, y:(e.clientY-r.top)*cv.height/r.height }; }
    let drag=null, moved=false;
    cv.addEventListener("pointerdown", function(e){
      if(e.button===2) return;                       // right button handled by contextmenu
      cv.setPointerCapture(e.pointerId); drag=px(e); moved=false; orbit=false; land=null;
    });
    cv.addEventListener("pointermove", function(e){
      if(!drag) return; const p=px(e);
      if(Math.abs(p.x-drag.x)+Math.abs(p.y-drag.y) > 3) moved=true;   // 3px threshold
      cam.yaw += (p.x-drag.x)*0.007;
      cam.pitch = Math.max(-1.3, Math.min(1.3, cam.pitch + (p.y-drag.y)*0.005));
      drag=p; draw();
    });
    cv.addEventListener("pointerup", function(e){
      const p=px(e); if(!moved) leftClick(p.x, p.y); drag=null;       // click = select+bounce
    });
    cv.addEventListener("pointercancel", function(){ drag=null; });
    cv.addEventListener("wheel", function(e){
      e.preventDefault();
      cam.k = Math.min(ZMAX, Math.max(ZMIN, cam.k*Math.exp(-e.deltaY*0.0016)));  // sky↔ground zoom
      draw();
    }, { passive:false });
    cv.addEventListener("contextmenu", function(e){
      e.preventDefault(); const p=px(e); rightClick(e, p.x, p.y);
    });
    if(global.document) global.document.addEventListener("click", function(){
      if(menuEl) menuEl.style.display="none";
    });

    /* ── optional land-from-sky ── */
    api.landFromSky = function(targetVec, done){
      const from={yaw:cam.yaw,pitch:cam.pitch,k:cam.k};
      const to={yaw:0.4, pitch:0.5, k:Math.min(ZMAX,2.4)};
      cam.pitch=1.35; cam.k=0.62;                      // start high in the sky
      const t0=Date.now(), dur=1500;
      land={from,to,t0,dur,done};
      logFn("⤓ landing from the sky","gold");
      if(REDUCED){ cam.yaw=to.yaw; cam.pitch=to.pitch; cam.k=to.k; land=null; if(done)done(); draw(); }
    };
    api.toggleOrbit = function(){ orbit=!orbit; return orbit; };

    /* ── animation loop ── */
    function tick(){
      tg++;
      if(land){
        const t=Math.min(1,(Date.now()-land.t0)/land.dur);
        const e=t<0.5?2*t*t:1-Math.pow(-2*t+2,2)/2;
        cam.yaw=1.35*(1-e)+land.to.yaw*e; // simple ease toward target
        cam.pitch=1.35*(1-e)+land.to.pitch*e;
        cam.k=0.62*(1-e)+land.to.k*e;
        draw(); if(t>=1){ if(land.done)land.done(); land=null; }
      } else if(orbit){ cam.yaw+=0.004; draw(); }
      else if(particles.some(p=>p.bounce>0.01)) draw();
      if(!REDUCED) global.requestAnimationFrame(tick);
    }
    draw();
    if(!REDUCED) global.requestAnimationFrame(tick); else draw();
    return api;
  }

  global.NasaruStyle = { create:create };
})(typeof window!=="undefined"?window:globalThis);
