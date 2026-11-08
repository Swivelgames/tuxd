//! Effective input context. Modal owners mask workspace scopes without bubbling.
use super::App;
impl App {
	pub fn input_scope(&self) -> crate::shortcuts::InputScope {
		if let Some(review) = &self.import_review {
			return review.view.input_scope();
		}
		if let Some(scope) = self.files.input_scope() {
			return scope;
		}
		self.workspace.input_scope()
	}
}
