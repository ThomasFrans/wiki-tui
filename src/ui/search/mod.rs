use crate::{
    config::Config,
    ui::utils::display_error,
    wiki::search::{Search, SearchResult},
};

use anyhow::Context;
use cursive::Cursive;

use super::utils::{display_dialog, display_message};

pub mod bar_popup;
mod display;
mod select;

/// Callback that searches for a given query and adds the results to a new layer
/// Displays any error that occurred and aborts the search (does not crash)
pub fn on_search(siv: &mut Cursive, query: &str) {
    let config = Config::from_siv(siv);

    // search for the query
    let search = match Search::builder()
        .query(query)
        .endpoint(config.borrow().api_config.url())
        .language(config.borrow().api_config.language.clone())
        .search()
        .with_context(|| format!("failed to search for '{}'", query))
    {
        Ok(search) => search,
        Err(error) => {
            warn!("{:?}", error);
            display_error(siv, error);
            return;
        }
    };

    // if we've found no results, display the appropriate message
    if search.is_empty() && search.suggestion().is_none() {
        warn!("could'nt find any search results and no query suggestion was given");
        display_message(
            siv,
            "Warning",
            &format!("Couldn't find any results for '{}'", query),
        );
        return;
    }

    // if we've found no results, but have a suggestion for another query, display a dialog
    if search.is_empty() && search.suggestion().is_some() {
        info!(
            "no results are available, suggesting a new query, '{}'",
            search.suggestion().unwrap()
        );

        display_dialog(
            siv,
            "Information",
            &format!(
                "No results for '{}' were found. Do you want to search for '{}' instead?",
                query,
                search.suggestion().unwrap()
            ),
            move |siv| on_search(siv, search.suggestion().unwrap()),
        );
        return;
    }

    // display the found search results
    // TODO:
    // - Why do we need 'query' when displaying search results?
    // - Why aren't they already in the `Search` struct?
    // - When suggesting a query, we need to pass the endpoint to `on_search`
    if let Err(error) = display::display_search_results(siv, search)
        .with_context(|| format!("failed to display the search results for '{}'", query))
    {
        warn!("{:?}", error);
        display_error(siv, error);
    }
}

/// Generates and displays a preview of a given search result. It's used as a callback for the
/// search results view
fn on_result_select(siv: &mut Cursive, item: &SearchResult) {
    info!(
        "selecting the item '{}', page id: '{}'",
        item.title(),
        item.pageid()
    );

    let layer_len = siv.screen_mut().len();

    let search_result_preview_name = format!("search_result_preview-{}", layer_len);
    let search_result_info_name = format!("search_result_info-{}", layer_len);

    // generate and display the preview of the search result
    if let Err(error) = select::generate_and_display_preview(siv, item, &search_result_preview_name)
        .context("failed generating and displaying the preview")
    {
        // only log the error and don't display it
        warn!("{:?}", error);
    }

    // generate and display the info of the search result
    if let Err(error) = select::generate_and_display_info(siv, item, &search_result_info_name)
        .context("failed generating and displaying the info")
    {
        // only log the error and don't display it
        warn!("{:?}", error);
    }
}

/// Searches for more results at a given offset and adds them to the results view. It's a callback
/// for the continue button and displays an error if something went wrong
fn on_continue_submit(siv: &mut Cursive, search: Search) {
    let offset = match search.continue_offset() {
        Some(offset) => offset,
        None => return,
    };

    // continue the search and fetch more results
    let continue_search = match Search::builder()
        .query(search.query)
        .endpoint(search.endpoint)
        .language(search.language)
        .offset(offset)
        .search()
        .context("failed to fetch more search results")
    {
        Ok(search) => search,
        Err(error) => {
            warn!("{:?}", error);
            display_error(siv, error);
            return;
        }
    };

    // display the search results
    if let Err(error) = display::display_more_search_results(siv, continue_search)
        .context("failed displaying more search results")
    {
        warn!("{:?}", error);
        display_error(siv, error);
    }
}
