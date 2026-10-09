#include <stdbool.h>

extern bool storekit2_app_store_can_make_payments(void)
    __asm__("_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ")
    __attribute__((swiftcall, weak_import));

bool framework_storekit2_can_make_payments(void) {
    if (storekit2_app_store_can_make_payments == 0) {
        return false;
    }

    return storekit2_app_store_can_make_payments();
}
