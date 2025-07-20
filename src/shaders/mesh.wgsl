struct Vertex {
	@location(0) pos: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
}

struct Fragment {
	@builtin(position) proj_pos: vec4<f32>,
	@location(0) world_pos: vec4<f32>,
	@location(1) normal: vec3<f32>,
}

struct PointLight {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
	@location(2) strength: f32,
}

struct Object {
	@location(0) model: mat4x4<f32>,
	@location(2) metallic: f32,
}

struct Camera {
	@location(0) position: vec3<f32>,
	@location(1) view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> lights: array<PointLight, 1>;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> object: Object;

@vertex
fn vert_main(in: Vertex) -> Fragment {
	var frag: Fragment;
	frag.world_pos = object.model * vec4(in.pos, 1.0);
	frag.normal = mat3x3<f32>(object.model[0].xyz, object.model[1].xyz, object.model[2].xyz) * in.normal;
	frag.proj_pos = camera.view_proj * frag.world_pos;
	return frag;
}

fn diffuse(l: vec3<f32>, n: vec3<f32>) -> f32 {
	return clamp(dot(l, n), 0.0, 1.0);
}

fn specular(l: vec3<f32>, v: vec3<f32>, n: vec3<f32>) -> f32 {
	let r = reflect(-l, n);
	return clamp(dot(r, v), 0.0, 1.0);
}

@fragment
fn frag_main(in: Fragment) -> @location(0) vec4<f32> {
	let n = normalize(in.normal);
	var result: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
	for (var i = 0u; i < 1; i++) {
		let light = lights[0];
		let l = normalize(light.position - in.world_pos.xyz);
		let v = normalize(camera.position - in.world_pos.xyz);
		let r = distance(light.position, in.world_pos.xyz);
		result += mix(diffuse(l, n), specular(l, v, n), object.metallic) * light.color * light.strength * (1.0 / (r * r + 1.0));
	}

	return vec4<f32>(result, 1.0);
	//return vec4<f32>(mix(diffuse(l, n), specular(l, v, n), object.metallic) * light.color * light.strength * (1.0 / (r * r + 1.0)), 1.);
}
