use std::{collections::HashSet, sync::LazyLock};

use async_trait::async_trait;
use maud::{PreEscaped, html};
use numbat::{
    InterpreterResult, InterpreterSettings, Statement,
    markup::{FormatType, FormattedString, Markup},
    module_importer::BuiltinModuleImporter,
    pretty_print::PrettyPrint,
    resolver::CodeSource,
};

use crate::engine::{answers::AnswerEngine, scrapers::SearchContext};

pub struct NumbatAnswer;

#[async_trait]
impl AnswerEngine for NumbatAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        let query = search
            .query
            .strip_suffix("=")
            .unwrap_or(&search.query)
            .trim();

        if !is_potential_request(query) {
            return None;
        }

        let (statement, result) = evaluate(query)?;
        let result_markup = result.pretty_print();

        if result_markup.to_string().trim() == query {
            return None;
        }

        let result_html = markup_to_html(result_markup);
        let statement_html = markup_to_html(statement.pretty_print());

        Some(
            html! {
                p.answer-query { (PreEscaped(statement_html)) " =" }
                p.answer-result { (PreEscaped(result_html)) }
            }
            .into_string(),
        )
    }
}

pub static NUMBAT_CTX: LazyLock<numbat::Context> = LazyLock::new(|| {
    let mut ctx = numbat::Context::new(BuiltinModuleImporter {});
    let _ = ctx.interpret("use prelude", CodeSource::Internal).unwrap();
    let _ = ctx
        .interpret("use units::currencies", CodeSource::Internal)
        .unwrap();

    ctx.load_currency_module_on_demand(true);

    [
        ("kb", "kB"),
        ("kib", "KiB"),
        ("mb", "MB"),
        ("mib", "MiB"),
        ("gb", "GB"),
        ("gib", "GiB"),
        ("tb", "TB"),
        ("tib", "TiB"),
        ("pb", "PB"),
        ("pib", "PiB"),
    ]
    .iter()
    .for_each(|(alias, canonical)| {
        let _ = ctx.interpret(&format!("let {alias} = {canonical}"), CodeSource::Internal);
    });

    let mut unit_names = HashSet::new();
    for names in ctx.unit_names() {
        unit_names.extend(names.iter().map(|name| name.to_owned()));
    }

    for name in &unit_names {
        let name_lower = name.to_lowercase();
        if !unit_names.contains(&name_lower) {
            let _ = ctx.interpret(&format!("let {name_lower} = {name}"), CodeSource::Internal);
        }
    }

    ctx
});

fn evaluate(query: &str) -> Option<(Statement<'_>, numbat::value::Value)> {
    let mut ctx = NUMBAT_CTX.clone();

    let (statements, result) = match ctx.interpret_with_settings(
        &mut InterpreterSettings {
            print_fn: Box::new(move |_: &numbat::markup::Markup| {}),
        },
        query,
        CodeSource::Text,
    ) {
        Ok(r) => r,
        Err(_e) => return None,
    };

    let InterpreterResult::Value(result) = result else {
        return None;
    };

    Some((statements.into_iter().next_back()?, result))
}

fn markup_to_html(markup: numbat::markup::Markup) -> String {
    let markup = fix_markup(markup);

    let mut html = String::new();
    for FormattedString(_output_type, format_type, content) in markup.0 {
        let class = match format_type {
            FormatType::Value => "calc-constant syn-constant",
            FormatType::String => "calc-string syn-string",
            FormatType::Identifier => "calc-func syn-func",
            _ => "",
        };
        if class.is_empty() {
            html.push_str(&html! { (content) }.into_string());
        } else {
            html.push_str(
                &html! {
                    span.(class) { (content) }
                }
                .into_string(),
            );
        }
    }

    html
}

fn fix_markup(markup: numbat::markup::Markup) -> numbat::markup::Markup {
    let mut reordered_markup: Vec<FormattedString> = Vec::new();
    const LEFT_SIDE_UNITS: &[&str] = &["$", "€", "£", "¥"];
    for s in markup.0 {
        let FormattedString(_output_type, format_type, content) = s.clone();

        if format_type == FormatType::Unit && LEFT_SIDE_UNITS.contains(&&*content) {
            reordered_markup.pop_if(|m| m.1 == FormatType::Whitespace);
            reordered_markup.insert(reordered_markup.len() - 1, s);
        } else {
            reordered_markup.push(s);
        }
    }
    Markup(reordered_markup)
}

fn is_potential_request(query: &str) -> bool {
    if matches!(query.to_lowercase().as_str(), "pi" | "e" | "c") {
        return true;
    }

    if query.len() < 3 {
        return false;
    }

    if !query.chars().any(|c| c.is_numeric()) {
        return false;
    }

    if query.starts_with('"')
        && query.ends_with('"')
        && query.chars().filter(|c| *c == '"').count() == 2
    {
        return false;
    }

    true
}
