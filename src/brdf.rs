use consts;
use vector::Vector3;

// 完全拡散反射のcos項による重点サンプリング
// https://github.com/githole/edupt/blob/master/radiance.h
pub fn importance_sample_diffuse(random: (f64, f64), normal: Vector3) -> Vector3 {
    let up = if normal.x.abs() > consts::EPS { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) };
    let tangent = up.cross(&normal).normalize();
    let binormal = normal.cross(&tangent);// up,tangent は直交かつ正規化されているので、normalize 不要
    // θ,φは極座標系の偏角。cosθにより重点サンプリングをしたい
    // 任意の確率密度関数fを積分した累積分布関数Fの逆関数を一様乱数に噛ませれば、
    // 任意の確率密度を持つ確率変数を得ることができる（逆関数法）
    // - f(θ,φ) = cos(θ)/PI
    // - F(θ,φ) = ∬f(θ,φ) dθdφ = φ/2PI * (1 - (cosθ)^2)
    // - F(θ) = 1 - (cosθ)^2
    // - F(φ) = φ/2PI
    // Fの逆関数から、角度θ,φを求めることができるので、
    //float theta = asin(sqrt(Xi.y));// θは整理すると消去できるのでコメントアウト
    let phi = consts::PI2 * random.0;
    // サンプリング方向 result は極座標から直交座標への変換によって求められる
    // result = tangent * sin(theta) * cos(phi) + binormal * sin(theta) * sin(phi) + normal * cos(theta))
    // ここで、r = Xi.y と置くと、result を整理できる
    // - sin(theta) = sin(asin(sqrt(Xi.y))) = sqrt(Xi.y) = sqrt(r)
    // - cos(theta) = sqrt(1.0 - sin(theta) * sin(theta)) = sqrt(1.0 - r)
    let r = random.1;
    return (tangent * phi.cos() + binormal * phi.sin()) * r.sqrt() + normal * (1.0 - r).sqrt();
}

// Unreal Engine 4 で利用されている ImportanceSampleGGX を移植
// cos項による重点サンプリングのためのハーフベクトルを計算
// http://project-asura.com/blog/?p=3124
pub fn importance_sample_ggx(random: (f64, f64), normal: Vector3, roughness: f64) -> Vector3 {
    let a = roughness * roughness;
    let phi = consts::PI2 * random.0;
    let cos_theta = ((1.0 - random.1) / (1.0 + (a * a - 1.0) * random.1)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let h = Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);

    let up = if normal.x.abs() > consts::EPS { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) };
    let tangent_x = up.cross(&normal).normalize();
    let tangent_y = normal.cross(&tangent_x);

    // Tangent to world space
    return tangent_x * h.x + tangent_y * h.y + normal * h.z;
}
