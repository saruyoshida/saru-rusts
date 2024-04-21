use sim_config::*;

fn main() {
  // ======================================
  // シミュレーション設定
  let mut sim = SimConfig::new();
  // ======================================
  // 粒子フィルタ設定
  let mut pf = sim.particlefilter();
  // ======================================
  // 粒子生成
  pf.create_particles(&sim.param);
  // 制御入力作成
  let cmd = sim.make_cmd();
  // 繰返し観測
  for (i, u) in cmd.enumerate() {
    // シミュレータ位置更新
    sim.set_u(u).move_next();
    if i % sim.dstep == 0 {
      println!("--loop--------------");
      println!("pos:{:?}", sim.pos);
      println!("zs:{:?}", sim.zs());
      println!("zp:{:?}", sim.zp);
    }
    // 予測
    pf.set_u(u).predict();
    if i % sim.dstep == 0 {
      println!("pt_pre[0]:{:?}, w:{}", 
                   pf.pt[0], pf.wg[0]);
    }
    // 更新
    pf.update(sim.lms(), sim.zs());
    if i % sim.dstep == 0 {
      println!("pt_upd[0]:{:?}, w:{}", 
                    pf.pt[0], pf.wg[0]);
      println!("neff:{}", pf.neff());
    }
    // 再サンプリング判定
    if pf.neff() < sim.jval {
      // 再サンプリング
      pf.resample();
      if i % sim.dstep == 0 {
        println!("pt_res[0]:{:?}, w:{}", 
                    pf.pt[0], pf.wg[0]);
      }
    }
    // 平均、分散算出
    let (mu, var) = pf.estimate();
    if i % sim.dstep == 0 {
      println!("mu:{:?}", mu);
      println!("var:{:?}", var);
    }
  }
}