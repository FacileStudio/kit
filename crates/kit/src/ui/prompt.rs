use anyhow::{bail, Result};
use dialoguer::{Confirm, MultiSelect, Select};

use super::Ui;

impl Ui {
    /// Asks a yes/no question on the terminal. `yes` answers it; with no terminal, kit refuses.
    pub fn confirm(&self, question: &str, yes: bool) -> Result<bool> {
        if yes {
            return Ok(true);
        }
        if !self.interactive() {
            bail!("not on a terminal: pass --yes to go ahead");
        }
        Ok(Confirm::new()
            .with_prompt(question)
            .default(false)
            .interact()?)
    }

    /// Lets the user pick one item. Returns its index.
    pub fn choose(&self, title: &str, items: &[String]) -> Result<usize> {
        Ok(Select::new()
            .with_prompt(title)
            .items(items)
            .default(0)
            .interact()?)
    }

    /// Lets the user check off items, starting from `checked`. Returns the chosen indexes.
    pub fn pick(&self, title: &str, items: &[String], checked: &[bool]) -> Result<Vec<usize>> {
        Ok(MultiSelect::new()
            .with_prompt(title)
            .items(items)
            .defaults(checked)
            .interact()?)
    }
}
