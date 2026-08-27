/* ============================================================================
   BahyWay Šala House Standard — the ŠURINNU LEGEND (GL-VIZ-002 candidate)
   ----------------------------------------------------------------------------
   A bordered stele of numbered rows and state-cells, replacing prose legends.
   NAME: shurinnu — the divine EMBLEM-STANDARDS carved in registers upon a
   kudurru stone. The Kudurru (sealed earlier in KAKIGIS) is the stone in the
   field — the beacon landmarks of Najaf; the Shurinnu are the emblems in its
   registers — this panel. One name, one meaning, and the two hold the exact
   relation their namesakes held.
   Grammar (emblems in registers, never paragraphs):
     - GROUPS  = column bands (TRIBES | ORBITS | EVENTS ...), label rotated.
     - ROWS    = one citizen-class per row, numbered on the left.
     - CELLS   = squares per group column:
         state 0 = HOLLOW  (declared, inactive)
         state 1 = FILLED  (present/active)          — in the row's colour
         state 2 = PULSING (firing NOW)              — alarm rhythm
       frac (0..1) optionally part-fills a cell (counts as fill height).
     - CLICK   = the legend is a CONTROLLER: select/filter/fly, never décor.
     - HOVER/SELECT gloss = ONE line, drawn in the stele footer.
   Usage:
     var K = SurinnuLegend(ctx, spec);   // spec: see below
     K.draw(nowMs);                      // each frame
     var hit = K.hitTest(mx,my);         // {gi,ri} | null  → act on it
   Spec:
     { x,y, cell:13, title:'ŠĒTU COURT',
       groups:[ {label:'TRIBES', rows:[
                   {id:'RIB', label:'ALLEY·RIBBON', color:'#6ea0cd',
                    state:1, frac:1, gloss:'26×~5 cells · PASSABLE map'}, ...]},
                {label:'EVENTS', rows:[...]} ],
       footer: '' }                      // set to gloss on hover/click
   ========================================================================== */
function SurinnuLegend(g,spec){
  var CS=spec.cell||13,GAP=3,ROWH=CS+GAP;
  function groupX(gi){var x=spec.x+26;for(var q=0;q<gi;q++)x+=CS+18;return x;}
  function rowsMax(){var m=0;spec.groups.forEach(function(G){m=Math.max(m,G.rows.length);});return m;}
  function panelW(){return 26+spec.groups.length*(CS+18)+34;}
  function panelH(){return 34+rowsMax()*ROWH+26;}
  function draw(now){
    var t=now/1000,X=spec.x,Y=spec.y,W=panelW(),H=panelH();
    g.fillStyle='rgba(20,18,15,0.88)';g.fillRect(X,Y,W,H);
    g.strokeStyle='#c9a227';g.lineWidth=1.2;g.strokeRect(X,Y,W,H);
    g.fillStyle='#c9a227';g.font='9.5px monospace';g.textAlign='left';
    g.fillText(spec.title||'ŠURINNU',X+8,Y+13);
    var mr=rowsMax();
    for(var r=0;r<mr;r++){
      g.fillStyle='rgba(143,134,114,0.85)';g.font='8.5px monospace';g.textAlign='right';
      g.fillText(String(r+1),X+20,Y+30+r*ROWH+CS-3);}
    spec.groups.forEach(function(G,gi){
      var gx=groupX(gi);
      g.save();g.translate(gx+CS+9,Y+30+((G.rows.length*ROWH))/2);
      g.rotate(Math.PI/2);
      g.fillStyle='rgba(143,134,114,0.9)';g.font='8.5px monospace';g.textAlign='center';
      g.fillText(G.label,0,0);g.restore();
      G.rows.forEach(function(R,ri){
        var cy=Y+30+ri*ROWH,col=R.color||'#e8e0cf';
        g.strokeStyle=col;g.lineWidth=1.3;
        g.strokeRect(gx,cy,CS,CS);
        var f=(R.state>=1)?(R.frac==null?1:Math.max(0,Math.min(1,R.frac))):0;
        if(R.state===2){var pl=0.45+0.55*Math.abs(Math.sin(t*4));
          g.globalAlpha=pl;f=1;}
        if(f>0){g.fillStyle=col;
          g.fillRect(gx+1.5,cy+1.5+(CS-3)*(1-f),CS-3,(CS-3)*f);}
        g.globalAlpha=1;
        if(R.sel){g.strokeStyle='#e8e0cf';g.lineWidth=1.6;
          g.strokeRect(gx-2.5,cy-2.5,CS+5,CS+5);}
      });});
    g.fillStyle='rgba(232,224,207,0.85)';g.font='8.5px monospace';g.textAlign='left';
    g.fillText((spec.footer||'').slice(0,Math.floor((W-14)/5.1)),X+8,Y+H-9);}
  function hitTest(mx,my){
    for(var gi=0;gi<spec.groups.length;gi++){
      var gx=groupX(gi);
      for(var ri=0;ri<spec.groups[gi].rows.length;ri++){
        var cy=spec.y+30+ri*ROWH;
        if(mx>=gx-2&&mx<=gx+CS+2&&my>=cy-2&&my<=cy+CS+2)return {gi:gi,ri:ri};}}
    return null;}
  return {draw:draw,hitTest:hitTest,panelW:panelW,panelH:panelH,spec:spec};
}
if(typeof module!=='undefined')module.exports=SurinnuLegend;
