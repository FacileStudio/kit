use kit_core::facile::Reporter;

use super::Ui;

impl Reporter for Ui {
    fn step(&self, message: &str) {
        Ui::step(self, message);
    }

    fn success(&self, message: &str) {
        Ui::success(self, message);
    }

    fn warn(&self, message: &str) {
        Ui::warn(self, message);
    }

    fn hint(&self, message: &str) {
        Ui::hint(self, message);
    }
}
