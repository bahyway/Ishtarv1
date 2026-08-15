// ============================================================================
// pdm_modeler.h — DubSar PDM (Particle Data Modeling) · GDExtension prototype
// BahyWay.Ecosystem v4.0 · PDM Modeler lens (GL-DST-001)
// Simplicial authoring + Z2 Betti DRAFT diagnostics + GA rotor orbit preview
// Law: draft math only — sealing routes to GeoEngine/Lamassu over HEPT.
// ============================================================================
#ifndef DUBSAR_PDM_MODELER_H
#define DUBSAR_PDM_MODELER_H

#include <godot_cpp/classes/node3d.hpp>
#include <godot_cpp/classes/rendering_device.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/variant/packed_string_array.hpp>
#include <vector>
#include <string>
#include <cstdint>

namespace godot {

class PDMModeler : public Node3D {
    GDCLASS(PDMModeler, Node3D)

public:
    // ---- Authoring API (the simplicial skeleton) ---------------------------
    void define_field(const String &name, const String &type,
                      const String &w5h2_role);                 // 0-simplex
    void link_fields(const String &a, const String &b);         // 1-simplex
    void add_composite(const PackedStringArray &field_names);   // 2-simplex
                                                                // (fan-split
                                                                // if >3)
    void expect_field(const String &name);  // declared expectation → gap
                                            // detection (HeptaMap Gap)

    // ---- TDA draft diagnostics --------------------------------------------
    // Returns {b0, b1, b2, gaps:[...], verdict:"HEALTHY|HOLD|REVIEW"}
    Dictionary compute_draft_diagnostics();

    // ---- Geometric Algebra orbit ------------------------------------------
    // Orbit plane bivector B = e_axis_a ^ e_axis_b of Hepta Space (0..6).
    void set_orbit_bivector(int axis_a, int axis_b);
    // RD compute pass: place `count` preview particles by rotor sandwich.
    void spawn_orbit_preview(int count);
    void advance_orbit(double delta);       // re-dispatch with new theta

    // ---- The contract ------------------------------------------------------
    // JSON tablet: fields, relations, composites, betti, gaps, bivector,
    // plus the HeptaScript contract string. Draft-sealed; real seal via HEPT.
    String emit_shape_tablet() const;

protected:
    static void _bind_methods();

private:
    struct Field { std::string name, type, role; };
    std::vector<Field> fields;
    std::vector<std::pair<int,int>> edges;          // vertex index pairs
    std::vector<std::array<int,3>> triangles;       // composite fans
    std::vector<std::string> expected;              // for gap detection

    int biv_a = 1, biv_b = 5;                       // default e1^e5

    // Z2 boundary-matrix rank via bitset Gaussian elimination (draft only).
    static int rank_z2(std::vector<std::vector<uint64_t>> &rows, int ncols);
    int find_field(const String &name) const;

    // RenderingDevice plumbing (compute pipeline for ga_orbit.comp.spv)
    RenderingDevice *rd = nullptr;
    RID shader, pipeline, particle_buf, uniform_set;
    int preview_count = 0;
    double theta = 0.0;
    void ensure_pipeline();
};

} // namespace godot
#endif
