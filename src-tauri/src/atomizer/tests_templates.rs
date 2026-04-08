use super::templates::load_templates;
use minijinja::context;

#[test]
fn load_templates_succeeds() {
    assert!(load_templates().is_ok());
}

#[test]
fn summarize_template_renders_plan_content() {
    let env = load_templates().unwrap();
    let tmpl = env.get_template("summarize").unwrap();
    let rendered = tmpl
        .render(context! { plan_content => "Feature: add login" })
        .unwrap();
    assert!(rendered.contains("Feature: add login"));
}

#[test]
fn chunk_template_renders_condensed_plan() {
    let env = load_templates().unwrap();
    let tmpl = env.get_template("chunk").unwrap();
    let rendered = tmpl
        .render(context! { condensed_plan => "Section A covers auth" })
        .unwrap();
    assert!(rendered.contains("Section A covers auth"));
}

#[test]
fn stories_template_renders_section_data() {
    let env = load_templates().unwrap();
    let tmpl = env.get_template("stories").unwrap();
    let rendered = tmpl
        .render(context! {
            project_name => "MyProject",
            section_title => "Auth",
            section_content => "Implement JWT login"
        })
        .unwrap();
    assert!(rendered.contains("MyProject"));
    assert!(rendered.contains("Auth"));
    assert!(rendered.contains("Implement JWT login"));
}

#[test]
fn merge_template_renders_project_and_stories() {
    let env = load_templates().unwrap();
    let tmpl = env.get_template("merge").unwrap();
    let rendered = tmpl
        .render(context! {
            project_name => "MyProject",
            generated_at => "2026-01-01T00:00:00Z",
            stories_json => "[]"
        })
        .unwrap();
    assert!(rendered.contains("MyProject"));
    assert!(rendered.contains("2026-01-01T00:00:00Z"));
}
