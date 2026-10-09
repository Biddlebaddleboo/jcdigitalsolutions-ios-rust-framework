#ifndef FRAMEWORK_IOS_MAPS_H
#define FRAMEWORK_IOS_MAPS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef double FrameworkIosMapsDistanceMeters;

/*
 * This header is opt-in through the framework-c-api Cargo feature `ios-maps`.
 * It exposes only caller-supplied MapKit geometry: coordinate/map-point
 * conversion and surface distance. Geographic input uses latitude
 * -90..=90 and longitude -180..=180 degrees. Map-point x/y values must be
 * finite and use MapKit projection units, not meters or screen points.
 *
 * All required outputs must be non-null. Each non-null output is initialized
 * to zero before validation or platform work. The two-output functions require
 * distinct, non-overlapping writable double slots. Non-finite or out-of-range
 * input returns FRAMEWORK_STATUS_INVALID_ARGUMENT. A valid request returns
 * FRAMEWORK_STATUS_OK only when MapKit returns a finite, in-range result;
 * otherwise it returns FRAMEWORK_STATUS_UNAVAILABLE. On non-iOS targets,
 * valid inputs return FRAMEWORK_STATUS_UNSUPPORTED and outputs remain zero.
 * A caught Rust panic returns FRAMEWORK_STATUS_PANIC with outputs zero.
 *
 * The wrapped MapKit geometry calls have an iOS 4.0 API floor. The feature
 * adds no main-thread rule, native object or handle, map UI, user-location
 * access, permission request, map data, or network service.
 */
FrameworkStatus framework_ios_maps_map_point_for_coordinate(
    double latitude_degrees,
    double longitude_degrees,
    double *out_x,
    double *out_y);

FrameworkStatus framework_ios_maps_coordinate_for_map_point(
    double x,
    double y,
    double *out_latitude_degrees,
    double *out_longitude_degrees);

FrameworkStatus framework_ios_maps_meters_between_map_points(
    double first_x,
    double first_y,
    double second_x,
    double second_y,
    FrameworkIosMapsDistanceMeters *out_meters);

#ifdef __cplusplus
}
#endif

#endif
