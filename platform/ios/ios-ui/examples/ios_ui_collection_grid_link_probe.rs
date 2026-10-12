#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{
    IosCollectionGridEvent, IosCollectionGridInsets, IosCollectionGridItem,
    IosCollectionGridLayout, IosCollectionGridSection, IosUiBackend,
};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let frame = Frame::new(0.0, 0.0, 320.0, 480.0).expect("valid collection frame");
    let Ok(mut parent) = backend.create_view(frame) else {
        return;
    };
    let sections = vec![
        IosCollectionGridSection::new(
            1,
            IosCollectionGridLayout::FixedGrid {
                columns: 2,
                item_height: 96.0,
                row_spacing: 8.0,
                column_spacing: 8.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            vec![
                IosCollectionGridItem::new(101, "Card").with_detail("Detail"),
                IosCollectionGridItem::metric(102, "Trips", "12").with_detail("This week"),
                IosCollectionGridItem::action(103, "Navigate").with_enabled(false),
                IosCollectionGridItem::toggle(u64::MAX, "Available", true),
            ],
        ),
        IosCollectionGridSection::new(
            2,
            IosCollectionGridLayout::AdaptiveGrid {
                minimum_item_width: 140.0,
                item_height: 96.0,
                maximum_columns: 4,
                row_spacing: 8.0,
                column_spacing: 8.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            vec![IosCollectionGridItem::new(201, "Adaptive")],
        ),
        IosCollectionGridSection::new(
            3,
            IosCollectionGridLayout::HorizontalCarousel {
                item_width: 180.0,
                item_height: 96.0,
                item_spacing: 12.0,
                insets: IosCollectionGridInsets::ZERO,
            },
            vec![IosCollectionGridItem::action(301, "Open").with_detail("Related item")],
        ),
    ];
    let Ok(mut grid) =
        backend.add_collection_grid(&mut parent, frame, sections, |event| match event {
            IosCollectionGridEvent::Selected { item_id } => {
                core::hint::black_box(item_id);
            }
            IosCollectionGridEvent::ActionTriggered { item_id } => {
                core::hint::black_box(item_id);
            }
            IosCollectionGridEvent::ToggleChanged { item_id, is_on } => {
                core::hint::black_box((item_id, is_on));
            }
        })
    else {
        return;
    };
    let next = vec![IosCollectionGridSection::new(
        1,
        IosCollectionGridLayout::FixedGrid {
            columns: 2,
            item_height: 96.0,
            row_spacing: 8.0,
            column_spacing: 8.0,
            insets: IosCollectionGridInsets::ZERO,
        },
        vec![IosCollectionGridItem::metric(102, "Trips", "13")],
    )];
    let _ = grid.replace_snapshot(next, true);
    core::hint::black_box(grid.native_collection_view());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
