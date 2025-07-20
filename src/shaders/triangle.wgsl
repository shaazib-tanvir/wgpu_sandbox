struct VertInput {
	@location(0) in_pos: vec3<f32>,
	@location(1) in_color: vec3<f32>,
	@location(2) in_uv: vec2<f32>,
}
struct VertOutput {
    @builtin(position) vert_position: vec4<f32>,
	@location(0) vert_color: vec3<f32>,
	@location(1) vert_uv: vec2<f32>,
};

@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;

@vertex
fn vert_main(in: VertInput) -> VertOutput {
	var vert_output: VertOutput;
	vert_output.vert_position = vec4<f32>(in.in_pos, 1.);
	vert_output.vert_color = in.in_color;
	vert_output.vert_uv = in.in_uv;
	return vert_output;
}

@fragment
fn frag_main(frag_data: VertOutput) -> @location(0) vec4<f32> {
	let color = textureSample(t, s, frag_data.vert_uv).xyz;
	return vec4<f32>(color * frag_data.vert_color, 1.);
}
