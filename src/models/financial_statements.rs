use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct IncomeStatement {
    pub net_sales: Option<f64>,
    pub cost_of_sales: Option<f64>,
    pub gross_profit: Option<f64>,
    pub selling_general_admin: Option<f64>,
    pub operating_income: Option<f64>,
    pub interest_income_noi: Option<f64>,
    pub dividends_income_noi: Option<f64>,
    pub interest_and_dividends_income_noi: Option<f64>,
    pub purchase_discounts_noi: Option<f64>,
    pub rent_income_noi: Option<f64>,
    pub house_rent_income_noi: Option<f64>,
    pub other_noi: Option<f64>,
    pub non_operating_income: Option<f64>,
    pub sales_discounts_noe: Option<f64>,
    pub rent_cost_real_estate_noe: Option<f64>,
    pub other_noe: Option<f64>,
    pub non_operating_expenses: Option<f64>,
    pub ordinary_income: Option<f64>,
    pub gain_on_sales_of_noncurrent_assets_ei: Option<f64>,
    pub extraordinary_income: Option<f64>,
    pub income_before_income_taxes: Option<f64>,
    pub income_taxes_current: Option<f64>,
    pub income_taxes_deferred: Option<f64>,
    pub income_taxes: Option<f64>,
    pub income_before_minority_interests: Option<f64>,
    pub net_income: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BalanceSheet {
    pub assets: Assets,
    pub liabilities: Liabilities,
    pub equity: Equity,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Assets {
    pub cash_and_deposits: Option<f64>,
    pub notes_and_accounts_receivable_trade: Option<f64>,
    pub short_term_investment_securities: Option<f64>,
    pub merchandise: Option<f64>,
    pub property_plant_and_equipment: Option<f64>,
    pub intangible_assets: Option<f64>,
    pub investments_and_other_assets: Option<f64>,
    pub total_assets: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Liabilities {
    pub current_liabilities: Option<f64>,
    pub noncurrent_liabilities: Option<f64>,
    pub total_liabilities: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Equity {
    pub shareholders_equity: Option<f64>,
    pub valuation_and_translation_adjustments: Option<f64>,
    pub total_equity: Option<f64>,
}
