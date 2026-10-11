use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use framework_ui::Frame;
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_ui_kit::{UILayoutConstraintAxis, UIStackView, UIStackViewDistribution, UIView};

use crate::grid_policy::GridPolicy;
use crate::platform::{IosUiBackend, IosView};

/// A retained UIKit equal-column grid for caller-owned native views.
///
/// UIKit retains arranged child views. Keep any `ios-ui` handles that own child behavior alive
/// separately. The grid is eager and does not recycle or virtualize children.
pub struct IosGridView {
    native: Retained<UIStackView>,
    policy: GridPolicy,
    current_row: Option<Retained<UIStackView>>,
    current_placeholders: Vec<Retained<UIView>>,
    next_column: u8,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds an equal-column grid to a caller-owned container.
    ///
    /// Rows and columns use UIKit points. `columns` must be nonzero, and both spacing values must
    /// be finite and nonnegative. UIKit uses native horizontal rows with equal distribution; empty
    /// cells preserve the column widths of an incomplete final row. The caller owns frame sizing,
    /// parent placement, and any child behavior handles.
    pub fn add_grid_view(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        columns: u8,
        row_spacing: f64,
        column_spacing: f64,
    ) -> Result<IosGridView> {
        let policy = GridPolicy::new(columns, row_spacing, column_spacing)?;
        let marker = main_thread_marker()?;
        let native = UIStackView::new(marker);
        native.setFrame(CGRect::new(
            CGPoint::new(frame.x(), frame.y()),
            CGSize::new(frame.width(), frame.height()),
        ));
        native.setAxis(UILayoutConstraintAxis::Vertical);
        native.setSpacing(row_spacing as _);
        parent.native_view().addSubview(&native);
        Ok(IosGridView {
            native,
            policy,
            current_row: None,
            current_placeholders: Vec::new(),
            next_column: 0,
            _not_send: PhantomData,
        })
    }
}

impl IosGridView {
    /// Adds a caller-owned native view as the next row-major grid item.
    ///
    /// UIKit retains the view and places it in an equal-width column. An incomplete last row keeps
    /// empty trailing cells; those placeholders are noninteractive and are not accessibility
    /// elements. The caller retains ownership of any control action handle.
    pub fn add_arranged_subview(&mut self, view: &UIView) -> Result<()> {
        let marker = main_thread_marker()?;
        if self.current_row.is_none() {
            let mut placeholders = Vec::new();
            placeholders
                .try_reserve_exact(usize::from(self.policy.columns))
                .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
            let row = UIStackView::new(marker);
            row.setAxis(UILayoutConstraintAxis::Horizontal);
            row.setDistribution(UIStackViewDistribution::FillEqually);
            row.setSpacing(self.policy.column_spacing as _);
            for _ in 0..self.policy.columns {
                let placeholder = UIView::new(marker);
                placeholder.setUserInteractionEnabled(false);
                row.addArrangedSubview(&placeholder);
                placeholders.push(placeholder);
            }
            self.native.addArrangedSubview(&row);
            self.current_row = Some(row);
            self.current_placeholders = placeholders;
        }

        let index = usize::from(self.next_column);
        let placeholder = self.current_placeholders.remove(0);
        let row = self
            .current_row
            .as_ref()
            .ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        row.removeArrangedSubview(&placeholder);
        placeholder.removeFromSuperview();
        row.insertArrangedSubview_atIndex(view, index);
        self.next_column += 1;
        if self.next_column == self.policy.columns {
            self.next_column = 0;
            self.current_row = None;
            self.current_placeholders.clear();
        }
        Ok(())
    }

    /// Borrows the native stack view for caller-owned UIKit constraints and integration.
    pub fn native_grid_view(&self) -> &UIStackView {
        &self.native
    }
}

fn main_thread_marker() -> Result<MainThreadMarker> {
    MainThread::current()
        .map(MainThread::into_objc2)
        .ok_or_else(|| Error::new(ErrorKind::Unavailable))
}
