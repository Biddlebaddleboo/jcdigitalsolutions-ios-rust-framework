#import <PhotosUI/PhotosUI.h>
#import <UIKit/UIKit.h>

static Class photo_picker_controller_class(void) {
    return PHPickerViewController.class;
}

static SEL photo_picker_init_selector(void) {
    return @selector(initWithConfiguration:);
}

static SEL photo_picker_delegate_selector(void) {
    return @selector(picker:didFinishPicking:);
}

int main(void) {
    return photo_picker_controller_class() == Nil || photo_picker_init_selector() == NULL || photo_picker_delegate_selector() == NULL;
}
