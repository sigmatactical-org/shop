use askama::Template;

use crate::catalog::CatalogSku;
use crate::model::{Listing, ShopUser, format_price_cents, price_cents_to_form};
use sigma_theme::copyright_years;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    storefront_items: Vec<StorefrontRow>,
    admin_rows: Vec<AdminRow>,
    catalog_configured: bool,
    catalog_error: Option<String>,
    identity_users: Vec<ShopUser>,
    identity_configured: bool,
    identity_error: Option<String>,
    message: Option<String>,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "form.html")]
struct FormTemplate {
    listing: Option<Listing>,
    sku_id: String,
    price: String,
    featured: bool,
    visible: bool,
    sort_order: String,
    catalog_skus: Vec<CatalogSkuRef>,
    error: Option<String>,
    copyright_years: String,
}

pub struct StorefrontRow {
    pub sku_code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price_display: String,
    pub featured: bool,
    pub missing_catalog: bool,
}

pub struct AdminRow {
    pub listing: Listing,
    pub sku_code: String,
    pub name: String,
    pub price_display: String,
    pub visible_label: String,
    pub featured_label: String,
    pub missing_catalog: bool,
}

pub struct CatalogSkuRef {
    pub id: String,
    pub sku_code: String,
    pub name: String,
}

pub struct FormValues {
    pub sku_id: String,
    pub price: String,
    pub featured: bool,
    pub visible: bool,
    pub sort_order: String,
}

fn catalog_sku_refs(skus: &[CatalogSku]) -> Vec<CatalogSkuRef> {
    skus.iter()
        .filter(|s| s.active)
        .map(|s| CatalogSkuRef {
            id: s.id.clone(),
            sku_code: s.sku_code.clone(),
            name: s.name.clone(),
        })
        .collect()
}

fn resolve_sku<'a>(skus: &'a [CatalogSku], sku_id: &str) -> Option<&'a CatalogSku> {
    skus.iter().find(|s| s.id == sku_id)
}

fn storefront_rows(listings: &[Listing], skus: &[CatalogSku]) -> Vec<StorefrontRow> {
    listings
        .iter()
        .filter(|l| l.visible)
        .filter_map(|listing| {
            let sku = resolve_sku(skus, &listing.sku_id);
            if sku.is_none() && !skus.is_empty() {
                return None;
            }
            let (sku_code, name, description, category) = match sku {
                Some(s) if s.active => (
                    s.sku_code.clone(),
                    s.name.clone(),
                    s.description.clone(),
                    s.category.clone(),
                ),
                Some(_) => return None,
                None => (
                    listing.sku_id.clone(),
                    "Unknown item".to_string(),
                    None,
                    None,
                ),
            };
            Some(StorefrontRow {
                sku_code,
                name,
                description,
                category,
                price_display: format_price_cents(listing.price_cents),
                featured: listing.featured,
                missing_catalog: sku.is_none(),
            })
        })
        .collect()
}

fn admin_rows(listings: &[Listing], skus: &[CatalogSku]) -> Vec<AdminRow> {
    listings
        .iter()
        .map(|listing| {
            let sku = resolve_sku(skus, &listing.sku_id);
            let (sku_code, name) = match sku {
                Some(s) => (s.sku_code.clone(), s.name.clone()),
                None => (listing.sku_id.clone(), "—".to_string()),
            };
            AdminRow {
                listing: listing.clone(),
                sku_code,
                name,
                price_display: format_price_cents(listing.price_cents),
                visible_label: if listing.visible {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
                featured_label: if listing.featured {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
                missing_catalog: sku.is_none(),
            }
        })
        .collect()
}

fn values_from_listing(listing: &Listing) -> FormValues {
    FormValues {
        sku_id: listing.sku_id.clone(),
        price: price_cents_to_form(listing.price_cents),
        featured: listing.featured,
        visible: listing.visible,
        sort_order: listing.sort_order.to_string(),
    }
}

fn default_form_values() -> FormValues {
    FormValues {
        sku_id: String::new(),
        price: String::new(),
        featured: false,
        visible: true,
        sort_order: String::new(),
    }
}

fn render_form(
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
    values: FormValues,
) -> Result<String, askama::Error> {
    FormTemplate {
        listing,
        sku_id: values.sku_id,
        price: values.price,
        featured: values.featured,
        visible: values.visible,
        sort_order: values.sort_order,
        catalog_skus: catalog_sku_refs(catalog_skus),
        error,
        copyright_years: copyright_years(),
    }
    .render()
}

/// Inputs for rendering the shop index page.
pub struct IndexPageInput<'a> {
    pub listings: Vec<Listing>,
    pub catalog_skus: &'a [CatalogSku],
    pub catalog_configured: bool,
    pub catalog_error: Option<String>,
    pub identity_users: &'a [ShopUser],
    pub identity_configured: bool,
    pub identity_error: Option<String>,
    pub message: Option<String>,
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_index_html(input: IndexPageInput<'_>) -> Result<String, askama::Error> {
    IndexTemplate {
        storefront_items: storefront_rows(&input.listings, input.catalog_skus),
        admin_rows: admin_rows(&input.listings, input.catalog_skus),
        catalog_configured: input.catalog_configured,
        catalog_error: input.catalog_error,
        identity_users: input.identity_users.to_vec(),
        identity_configured: input.identity_configured,
        identity_error: input.identity_error,
        message: input.message,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_form_html(
    _listings: Vec<Listing>,
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
) -> Result<String, askama::Error> {
    let values = listing
        .as_ref()
        .map(values_from_listing)
        .unwrap_or_else(default_form_values);
    render_form(catalog_skus, listing, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_form_html_with_values(
    _listings: Vec<Listing>,
    catalog_skus: &[CatalogSku],
    listing: Option<Listing>,
    error: Option<String>,
    values: FormValues,
) -> Result<String, askama::Error> {
    render_form(catalog_skus, listing, error, values)
}
