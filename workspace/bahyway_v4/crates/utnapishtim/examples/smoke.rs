use utnapishtim::{ClientTopology, generator::UtnapishtimGenerator};

fn main() {
    let topo = ClientTopology {
        client_id: 7,
        client_name: "Najaf Cemetery".into(),
        tribes: ClientTopology::build_tribes(7, &["Graves", "Records", "Family"]),
        sealed_at: 1234567890,
    };
    let gen = UtnapishtimGenerator::new("/tmp/utnapishtim_smoke_out");
    let result = gen.generate(&topo).expect("generate must succeed");
    println!("html: {:?}", result.html_path);
    println!("godot_dir: {:?}", result.godot_dir);
    println!("manifest: {:?}", result.manifest_path);

    let html = std::fs::read_to_string(&result.html_path).unwrap();
    assert!(html.contains("Najaf Cemetery"));
    assert!(html.contains("<!DOCTYPE html>"));

    for f in ["project.godot", "orbital_manager.gd", "particle_node.gd", "tribe_config.gd", "enkidb_connector.gd"] {
        let p = result.godot_dir.join(f);
        let content = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("missing {f}: {e}"));
        assert!(!content.is_empty(), "{f} is empty");
        println!("  {f}: {} bytes OK", content.len());
    }

    let manifest = std::fs::read_to_string(&result.manifest_path).unwrap();
    assert!(manifest.contains("client_id"));

    println!("SMOKE TEST PASSED");
}
