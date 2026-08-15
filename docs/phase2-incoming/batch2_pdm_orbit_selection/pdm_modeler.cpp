// ============================================================================
// pdm_modeler.cpp — DubSar PDM prototype implementation
// Draft diagnostics only; sealing routes to GeoEngine/Lamassu (GL-DST-001 §4).
// ============================================================================
#include "pdm_modeler.h"
#include <godot_cpp/classes/rendering_server.hpp>
#include <godot_cpp/classes/rd_shader_spirv.hpp>
#include <godot_cpp/classes/rd_shader_file.hpp>
#include <godot_cpp/classes/rd_uniform.hpp>
#include <godot_cpp/classes/file_access.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <algorithm>
#include <sstream>

using namespace godot;

// ---------------------------------------------------------------- authoring
void PDMModeler::define_field(const String &name, const String &type,
                              const String &role) {
    fields.push_back({name.utf8().get_data(), type.utf8().get_data(),
                      role.utf8().get_data()});
}

int PDMModeler::find_field(const String &name) const {
    std::string n = name.utf8().get_data();
    for (size_t i = 0; i < fields.size(); ++i)
        if (fields[i].name == n) return (int)i;
    return -1;
}

void PDMModeler::link_fields(const String &a, const String &b) {
    int ia = find_field(a), ib = find_field(b);
    if (ia < 0 || ib < 0 || ia == ib) return;
    if (ia > ib) std::swap(ia, ib);
    if (std::find(edges.begin(), edges.end(),
                  std::make_pair(ia, ib)) == edges.end())
        edges.push_back({ia, ib});
}

void PDMModeler::add_composite(const PackedStringArray &names) {
    // Fan-split any k-composite into triangles anchored at names[0];
    // also ensure all boundary edges exist (a composite implies relations).
    std::vector<int> idx;
    for (int i = 0; i < names.size(); ++i) {
        int f = find_field(names[i]);
        if (f >= 0) idx.push_back(f);
    }
    if (idx.size() < 3) { // a 2-field composite is just an edge
        if (idx.size() == 2) link_fields(names[0], names[1]);
        return;
    }
    for (size_t k = 1; k + 1 < idx.size(); ++k) {
        std::array<int,3> t = {idx[0], idx[k], idx[k+1]};
        std::sort(t.begin(), t.end());
        triangles.push_back(t);
        // boundary edges
        auto add_e = [&](int a, int b){
            if (a > b) std::swap(a,b);
            if (std::find(edges.begin(), edges.end(),
                          std::make_pair(a,b)) == edges.end())
                edges.push_back({a,b});
        };
        add_e(t[0],t[1]); add_e(t[0],t[2]); add_e(t[1],t[2]);
    }
}

void PDMModeler::expect_field(const String &name) {
    expected.push_back(name.utf8().get_data());
}

// ------------------------------------------------------- Z2 linear algebra
int PDMModeler::rank_z2(std::vector<std::vector<uint64_t>> &rows, int ncols) {
    int rank = 0, nwords = (ncols + 63) / 64;
    for (int col = 0; col < ncols && rank < (int)rows.size(); ++col) {
        int w = col >> 6; uint64_t bit = 1ull << (col & 63);
        int pivot = -1;
        for (int r = rank; r < (int)rows.size(); ++r)
            if (rows[r][w] & bit) { pivot = r; break; }
        if (pivot < 0) continue;
        std::swap(rows[rank], rows[pivot]);
        for (int r = 0; r < (int)rows.size(); ++r)
            if (r != rank && (rows[r][w] & bit))
                for (int q = 0; q < nwords; ++q) rows[r][q] ^= rows[rank][q];
        ++rank;
    }
    return rank;
}

Dictionary PDMModeler::compute_draft_diagnostics() {
    const int V = (int)fields.size();
    const int E = (int)edges.size();
    const int T = (int)triangles.size();

    // ∂1 : rows = edges over vertex columns
    std::vector<std::vector<uint64_t>> d1(E,
        std::vector<uint64_t>((V + 63) / 64, 0));
    for (int e = 0; e < E; ++e) {
        d1[e][edges[e].first  >> 6] |= 1ull << (edges[e].first  & 63);
        d1[e][edges[e].second >> 6] |= 1ull << (edges[e].second & 63);
    }
    // ∂2 : rows = triangles over edge columns
    auto edge_index = [&](int a, int b)->int{
        if (a > b) std::swap(a,b);
        for (int e = 0; e < E; ++e)
            if (edges[e].first == a && edges[e].second == b) return e;
        return -1;
    };
    std::vector<std::vector<uint64_t>> d2(T,
        std::vector<uint64_t>((E + 63) / 64, 0));
    for (int t = 0; t < T; ++t) {
        int ids[3] = { edge_index(triangles[t][0], triangles[t][1]),
                       edge_index(triangles[t][0], triangles[t][2]),
                       edge_index(triangles[t][1], triangles[t][2]) };
        for (int k = 0; k < 3; ++k)
            if (ids[k] >= 0) d2[t][ids[k] >> 6] |= 1ull << (ids[k] & 63);
    }
    int r1 = rank_z2(d1, V);
    int r2 = rank_z2(d2, E);
    int b0 = V - r1;
    int b1 = E - r1 - r2;
    int b2 = std::max(0, T - r2);   // r3 = 0 (no tetrahedra authored)

    // HeptaMap Gaps: expected fields absent from the complex
    PackedStringArray gaps;
    for (auto &g : expected)
        if (find_field(String(g.c_str())) < 0)
            gaps.push_back(String(g.c_str()));

    String verdict = "HEALTHY";
    if (b0 > 1 || gaps.size() > 0) verdict = "HOLD";      // islands or gaps
    else if (b1 > 0 || b2 > 0)     verdict = "REVIEW";    // loops or voids

    Dictionary d;
    d["b0"] = b0; d["b1"] = b1; d["b2"] = b2;
    d["gaps"] = gaps; d["verdict"] = verdict;
    d["note"] = "DRAFT diagnostics — seal via HEPT: GeoEngine verifies, "
                "Lamassu certifies (GL-DST-001 §4).";
    return d;
}

// ------------------------------------------------- GA orbit preview (RD)
void PDMModeler::set_orbit_bivector(int a, int b) {
    biv_a = CLAMP(a, 0, 6); biv_b = CLAMP(b, 0, 6);
    if (biv_a == biv_b) biv_b = (biv_a + 1) % 7;
}

void PDMModeler::ensure_pipeline() {
    if (rd) return;
    rd = RenderingServer::get_singleton()->get_rendering_device();
    Ref<RDShaderFile> sf; sf.instantiate();
    // ga_orbit.comp.glsl compiled offline: glslc → ga_orbit.comp.spv,
    // wrapped as .glsl shader file or loaded raw:
    PackedByteArray spv =
        FileAccess::get_file_as_bytes("res://shaders/ga_orbit.comp.spv");
    Ref<RDShaderSPIRV> sp; sp.instantiate();
    sp->set_stage_bytecode(RenderingDevice::SHADER_STAGE_COMPUTE, spv);
    shader   = rd->shader_create_from_spirv(sp);
    pipeline = rd->compute_pipeline_create(shader);
}

void PDMModeler::spawn_orbit_preview(int count) {
    ensure_pipeline();
    preview_count = count;
    // Buffer layout: per particle — vec4 hepta7_lo (e0..e3),
    // vec4 hepta7_hi (e4..e6 + phase). 32 bytes.
    PackedByteArray init; init.resize(count * 32);
    // seed radial band + phase deterministically (rehearsal seed)
    uint32_t s = 0x9E3779B9u;
    auto nrand = [&](){ s ^= s<<13; s ^= s>>17; s ^= s<<5; return s; };
    float *f = reinterpret_cast<float*>(init.ptrw());
    for (int i = 0; i < count; ++i) {
        for (int k = 0; k < 7; ++k) f[i*8+k] = 0.0f;
        float r = 0.35f + 0.15f * (nrand() / 4294967296.0f);
        f[i*8 + biv_a] = r;                       // start vector in plane
        f[i*8 + 7]     = 6.28318f * (nrand() / 4294967296.0f); // phase
    }
    particle_buf = rd->storage_buffer_create(init.size(), init);

    Ref<RDUniform> u; u.instantiate();
    u->set_uniform_type(RenderingDevice::UNIFORM_TYPE_STORAGE_BUFFER);
    u->set_binding(0); u->add_id(particle_buf);
    TypedArray<RDUniform> us; us.push_back(u);
    uniform_set = rd->uniform_set_create(us, shader, 0);
    theta = 0.0;
    advance_orbit(0.0);
}

void PDMModeler::advance_orbit(double delta) {
    if (!rd || preview_count == 0) return;
    theta += delta * 0.4;
    // push constants: theta, plane axes, count
    PackedByteArray pc; pc.resize(16);
    float  *pf = reinterpret_cast<float*>(pc.ptrw());
    int32_t*pi = reinterpret_cast<int32_t*>(pc.ptrw());
    pf[0] = (float)theta; pi[1] = biv_a; pi[2] = biv_b; pi[3] = preview_count;

    int64_t cl = rd->compute_list_begin();
    rd->compute_list_bind_compute_pipeline(cl, pipeline);
    rd->compute_list_bind_uniform_set(cl, uniform_set, 0);
    rd->compute_list_set_push_constant(cl, pc, pc.size());
    rd->compute_list_dispatch(cl, (preview_count + 255) / 256, 1, 1);
    rd->compute_list_end();
    // Positions buffer is then consumed by the BigRingRenderer draw pass
    // (MultiMesh sampler or indirect draw — build guide Steps 4/10).
}

// ---------------------------------------------------------- the contract
String PDMModeler::emit_shape_tablet() const {
    // Draft-sealed JSON + HeptaScript; real AkkadianSeal happens engine-side.
    std::ostringstream j;
    j << "{ \"shape\": { \"fields\": [";
    for (size_t i = 0; i < fields.size(); ++i)
        j << (i ? "," : "") << "{\"name\":\"" << fields[i].name
          << "\",\"type\":\"" << fields[i].type
          << "\",\"w5h2\":\"" << fields[i].role << "\"}";
    j << "], \"relations\": " << edges.size()
      << ", \"composites\": " << triangles.size()
      << ", \"orbit_bivector\": \"e" << biv_a << "^e" << biv_b << "\" },\n";
    j << "  \"heptascript\": \"PRESENT SHAPE Draft\\n"
      << "  WHAT fields = " << fields.size()
      << " . relations = " << edges.size()
      << " . composites = " << triangles.size() << "\\n"
      << "  WHERE orbit = BIVECTOR e" << biv_a << "^e" << biv_b << "\\n"
      << "EMIT ShapeTablet SEAL AkkadianSeal\",\n"
      << "  \"seal\": \"DRAFT — route via HEPT for GeoEngine/Lamassu/G4\" }";
    return String(j.str().c_str());
}

// ----------------------------------------------------------------- binds
void PDMModeler::_bind_methods() {
    ClassDB::bind_method(D_METHOD("define_field","name","type","w5h2_role"),
                         &PDMModeler::define_field);
    ClassDB::bind_method(D_METHOD("link_fields","a","b"),
                         &PDMModeler::link_fields);
    ClassDB::bind_method(D_METHOD("add_composite","field_names"),
                         &PDMModeler::add_composite);
    ClassDB::bind_method(D_METHOD("expect_field","name"),
                         &PDMModeler::expect_field);
    ClassDB::bind_method(D_METHOD("compute_draft_diagnostics"),
                         &PDMModeler::compute_draft_diagnostics);
    ClassDB::bind_method(D_METHOD("set_orbit_bivector","axis_a","axis_b"),
                         &PDMModeler::set_orbit_bivector);
    ClassDB::bind_method(D_METHOD("spawn_orbit_preview","count"),
                         &PDMModeler::spawn_orbit_preview);
    ClassDB::bind_method(D_METHOD("advance_orbit","delta"),
                         &PDMModeler::advance_orbit);
    ClassDB::bind_method(D_METHOD("emit_shape_tablet"),
                         &PDMModeler::emit_shape_tablet);
}
