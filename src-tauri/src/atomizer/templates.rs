use crate::atomizer::AtomizerError;
use minijinja::Environment;

pub(super) fn load_templates() -> Result<Environment<'static>, AtomizerError> {
    let mut env = Environment::new();
    env.add_template_owned(
        "summarize",
        include_str!("../../templates/atomize_summarize.j2").to_string(),
    )
    .map_err(|err| AtomizerError::Template(err.to_string()))?;
    env.add_template_owned(
        "chunk",
        include_str!("../../templates/atomize_chunk.j2").to_string(),
    )
    .map_err(|err| AtomizerError::Template(err.to_string()))?;
    env.add_template_owned(
        "stories",
        include_str!("../../templates/atomize_stories.j2").to_string(),
    )
    .map_err(|err| AtomizerError::Template(err.to_string()))?;
    env.add_template_owned(
        "merge",
        include_str!("../../templates/atomize_merge.j2").to_string(),
    )
    .map_err(|err| AtomizerError::Template(err.to_string()))?;
    Ok(env)
}
