import re
import sys
from pathlib import Path


def function_span(source, name):
 pattern = re.search(r"\bfn\s+" + re.escape(name) + r"(?:<[^>]*>)?\s*\(", source)
 if pattern is None:
  raise ValueError("missing function " + name)
 opening = source.find("{", pattern.end())
 if opening < 0:
  raise ValueError("missing body for " + name)
 depth = 0
 for index in range(opening, len(source)):
  if source[index] == "{":
   depth += 1
  elif source[index] == "}":
   depth -= 1
   if depth == 0:
    return pattern.start(), index + 1
 raise ValueError("unterminated body for " + name)


def function_body(source, name):
 start, end = function_span(source, name)
 return source[start:end]


def replace_function(source, name, old, new):
 start, end = function_span(source, name)
 body = source[start:end]
 if old not in body:
  raise ValueError("negative fixture needle absent in " + name)
 return source[:start] + body.replace(old, new, 1) + source[end:]


def insert_function(source, name, marker, addition):
 start, end = function_span(source, name)
 body = source[start:end]
 if marker not in body:
  raise ValueError("negative fixture insertion point absent in " + name)
 return source[:start] + body.replace(marker, marker + "\n" + addition, 1) + source[end:]


def block_after(source, marker):
 start = source.find(marker)
 if start < 0:
  raise ValueError("missing block " + marker)
 opening = source.find("{", start + len(marker))
 if opening < 0:
  raise ValueError("missing block body " + marker)
 depth = 0
 for index in range(opening, len(source)):
  if source[index] == "{":
   depth += 1
  elif source[index] == "}":
   depth -= 1
   if depth == 0:
    return source[opening:index + 1]
 raise ValueError("unterminated block " + marker)


def check(source):
 errors = []
 def require(condition, label):
  if not condition:
   errors.append(label)

 functions = {}
 for name in ("start_authorization_request", "start_always_authorization_request", "start_current",
              "start_background_updates", "start_updates", "start_location_updates",
              "main_bundle_has_location_background_mode", "main_bundle_has_always_usage_descriptions",
              "main_bundle_requires_legacy_always_usage_description",
              "locationManager_didUpdateLocations", "complete_authorization_change"):
  try:
   functions[name] = function_body(source, name)
  except ValueError:
   functions[name] = ""
   errors.append("function " + name)

 selectors = (
  ("requestWhenInUseAuthorization()", "start_authorization_request"),
  ("requestAlwaysAuthorization()", "start_always_authorization_request"),
  ("requestLocation()", "start_current"),
  ("startUpdatingLocation()", "start_location_updates"),
  ("setAllowsBackgroundLocationUpdates(true)", "start_location_updates"),
 )
 for selector, owner in selectors:
  require(source.count(selector) == 1 and functions[owner].count(selector) == 1,
          "selector scope " + selector)

 always = functions["start_always_authorization_request"]
 eligibility = "if !should_request_always_authorization(authorization)"
 usage_check = "if !main_bundle_has_always_usage_descriptions()"
 always_call = "manager.requestAlwaysAuthorization()"
 require(eligibility in always, "Always request eligibility")
 require(usage_check in always and always_call in always and
         always.index(usage_check) < always.index(always_call), "Always config before request")
 require(eligibility in always and always_call in always and
         always.index(eligibility) < always.index(always_call),
         "Always request is conditional")

 usage = functions["main_bundle_has_always_usage_descriptions"]
 for key in ("NSLocationWhenInUseUsageDescription", "NSLocationAlwaysAndWhenInUseUsageDescription",
             "NSLocationAlwaysUsageDescription"):
  require(key in usage, "Always usage-key mapping")
 require("main_bundle_requires_legacy_always_usage_description()" in usage,
         "legacy usage-key deployment target")
 legacy_target = functions["main_bundle_requires_legacy_always_usage_description"]
 require('ns_string!("MinimumOSVersion")' in legacy_target and
         ".doubleValue()" in legacy_target and
         "requires_legacy_always_usage_description(minimum_version)" in legacy_target,
         "legacy usage-key deployment target parser")

 background = functions["start_background_updates"]
 require("UpdateAuthorizationRequirement::Background" in background and "Some(options)" in background,
         "background session requires Always")
 foreground = functions["start_updates"]
 require("UpdateAuthorizationRequirement::Foreground" in foreground and "None" in foreground,
         "foreground session stays foreground-only")

 updates = functions["start_location_updates"]
 mode_check = "if !main_bundle_has_location_background_mode()"
 background_setter = "manager.setAllowsBackgroundLocationUpdates(true)"
 try:
  background_branch = block_after(updates,
                                  "(UpdateAuthorizationRequirement::Background, Some(_)) =>")
 except ValueError:
  background_branch = ""
 require(mode_check in background_branch, "background plist preflight branch")
 require("if !updates_permitted(authorization, requirement)" in updates,
         "session authorization check")
 require(mode_check in updates and background_setter in updates and
         updates.index(mode_check) < updates.index(background_setter), "background plist preflight order")
 try:
  options = block_after(updates, "if let Some(options) = background_options")
 except ValueError:
  options = ""
 require("setDistanceFilter(objc2_core_location::kCLDistanceFilterNone)" in options and
         "setActivityType(options.activity_type.native())" in options and
         "setPausesLocationUpdatesAutomatically(options.pauses_automatically)" in options and
         background_setter in options, "background option mapping and setter scope")
 indicator_guard = "!objc2::available!(ios = 11.0, ..)"
 indicator_setter = "manager.setShowsBackgroundLocationIndicator(true)"
 indicator_option = "if options.shows_background_indicator"
 try:
  indicator_block = block_after(updates, indicator_option)
 except ValueError:
  indicator_block = ""
 require(indicator_option in updates and indicator_setter in indicator_block,
         "indicator option is respected")
 require(indicator_guard in updates and indicator_setter in updates and
         updates.index(indicator_guard) < updates.index(indicator_setter), "indicator availability")

 mode_parser = functions["main_bundle_has_location_background_mode"]
 for needle in ("UIBackgroundModes", "downcast::<NSArray<AnyObject>>()", "downcast_ref::<NSString>",
                'ns_string!("location")', "has_location_background_mode(Some(entries)"):
  require(needle in mode_parser, "UIBackgroundModes parser")

 callback = functions["locationManager_didUpdateLocations"]
 update_branch = callback[callback.find("DelegateState::Updates"):]
 ordered = ("let current_authorization =", "authorization_from_native(native_authorization_status())",
            "if authorization.observe(current_authorization)",
            "queue.push_batch([LocationUpdateKind::Error(permission_error())])",
            "if !authorization.permits(current_authorization)", "return;", "let result = catch_unwind")
 positions = [update_branch.find(needle) for needle in ordered]
 require(all(position >= 0 for position in positions) and positions == sorted(positions),
         "background downgrade event and fix suppression")
 authorization_callback = functions["complete_authorization_change"]
 require("DelegateState::Updates" in authorization_callback and
         "authorization.observe(authorization_from_native(raw_status))" in authorization_callback and
         "queue.push_batch([LocationUpdateKind::Error(permission_error())])" in authorization_callback,
         "authorization callback reports background downgrade")
 return errors


def verify_negative_fixtures(source):
 cases = (
  ("selector scope requestWhenInUseAuthorization()", "start_authorization_request", "manager.requestWhenInUseAuthorization()", "removed_request()"),
  ("selector scope requestAlwaysAuthorization()", "start_always_authorization_request", "manager.requestAlwaysAuthorization()", "removed_request()"),
  ("selector scope requestLocation()", "start_current", "manager.requestLocation()", "removed_request()"),
  ("selector scope startUpdatingLocation()", "start_location_updates", "manager.startUpdatingLocation()", "removed_update()"),
  ("selector scope setAllowsBackgroundLocationUpdates(true)", "start_location_updates", "manager.setAllowsBackgroundLocationUpdates(true)", "removed_background_enable()"),
  ("Always request eligibility", "start_always_authorization_request", "if !should_request_always_authorization(authorization)", ""),
  ("Always config before request", "start_always_authorization_request", "if !main_bundle_has_always_usage_descriptions()", ""),
  ("Always usage-key mapping", "main_bundle_has_always_usage_descriptions", "NSLocationWhenInUseUsageDescription", "RemovedWhenInUseUsageKey"),
  ("Always usage-key mapping", "main_bundle_has_always_usage_descriptions", "NSLocationAlwaysAndWhenInUseUsageDescription", "RemovedUsageKey"),
  ("Always usage-key mapping", "main_bundle_has_always_usage_descriptions", "NSLocationAlwaysUsageDescription", "RemovedLegacyUsageKey"),
  ("legacy usage-key deployment target", "main_bundle_has_always_usage_descriptions", "main_bundle_requires_legacy_always_usage_description()", "false"),
  ("legacy usage-key deployment target parser", "main_bundle_requires_legacy_always_usage_description", "MinimumOSVersion", "RemovedMinimumOSVersion"),
  ("legacy usage-key deployment target parser", "main_bundle_requires_legacy_always_usage_description", ".doubleValue()", "removed_version_parse()"),
  ("background session requires Always", "start_background_updates", "UpdateAuthorizationRequirement::Background", "UpdateAuthorizationRequirement::Foreground"),
  ("background session requires Always", "start_background_updates", "Some(options)", "None"),
  ("foreground session stays foreground-only", "start_updates", "UpdateAuthorizationRequirement::Foreground", "UpdateAuthorizationRequirement::Background"),
  ("session authorization check", "start_location_updates", "if !updates_permitted(authorization, requirement)", "if true"),
  ("background plist preflight order", "start_location_updates", "if !main_bundle_has_location_background_mode()", "if true"),
  ("background plist preflight branch", "start_location_updates", "if !main_bundle_has_location_background_mode()", "if true"),
  ("background option mapping and setter scope", "start_location_updates", "if let Some(options) = background_options", "if true"),
  ("background option mapping and setter scope", "start_location_updates", "setDistanceFilter(objc2_core_location::kCLDistanceFilterNone)", "removed_distance_filter()"),
  ("background option mapping and setter scope", "start_location_updates", "setActivityType(options.activity_type.native())", "removed_activity_type()"),
  ("background option mapping and setter scope", "start_location_updates", "setPausesLocationUpdatesAutomatically(options.pauses_automatically)", "removed_automatic_pausing()"),
  ("background option mapping and setter scope", "start_location_updates", "setAllowsBackgroundLocationUpdates(true)", "removed_background_enable()"),
  ("indicator availability", "start_location_updates", "!objc2::available!(ios = 11.0, ..)", "true"),
  ("indicator option is respected", "start_location_updates", "if options.shows_background_indicator", "if true"),
  ("UIBackgroundModes parser", "main_bundle_has_location_background_mode", "UIBackgroundModes", "RemovedBackgroundModes"),
  ("UIBackgroundModes parser", "main_bundle_has_location_background_mode", 'ns_string!("location")', 'ns_string!("audio")'),
  ("UIBackgroundModes parser", "main_bundle_has_location_background_mode", "downcast::<NSArray<AnyObject>>()", "removed_array_downcast()"),
  ("UIBackgroundModes parser", "main_bundle_has_location_background_mode", "downcast_ref::<NSString>", "removed_string_downcast"),
  ("background downgrade event and fix suppression", "locationManager_didUpdateLocations", "authorization_from_native(native_authorization_status())", "authorization_from_native(0)"),
  ("background downgrade event and fix suppression", "locationManager_didUpdateLocations", "if authorization.observe(current_authorization)", "if false"),
  ("background downgrade event and fix suppression", "locationManager_didUpdateLocations", "queue.push_batch([LocationUpdateKind::Error(permission_error())])", "removed_permission_event()"),
  ("background downgrade event and fix suppression", "locationManager_didUpdateLocations", "if !authorization.permits(current_authorization)", "if true"),
  ("background downgrade event and fix suppression", "locationManager_didUpdateLocations", "if !authorization.permits(current_authorization) {\n                        return;\n                    }", "if !authorization.permits(current_authorization) {\n                        drop(());\n                    }"),
  ("authorization callback reports background downgrade", "complete_authorization_change", "if authorization.observe(authorization_from_native(raw_status))", "if false"),
 )
 failures = []
 wrong_scope_cases = (
  ("selector scope startUpdatingLocation()", "start_current", "let native_accuracy =", "unsafe { manager.startUpdatingLocation() };"),
  ("selector scope requestLocation()", "start_location_updates", "if !updates_permitted(authorization, requirement)", "unsafe { manager.requestLocation() };"),
  ("selector scope requestAlwaysAuthorization()", "start_authorization_request", "let status = native_authorization_status();", "unsafe { manager.requestAlwaysAuthorization() };"),
 )
 for expected, function, marker, addition in wrong_scope_cases:
  try:
   mutated = insert_function(source, function, marker, addition)
  except ValueError as error:
   failures.append(str(error))
   continue
  if expected not in check(mutated):
   failures.append("wrong-scope mutation escaped " + expected)
 for expected, function, old, new in cases:
  try:
   mutated = replace_function(source, function, old, new)
  except ValueError as error:
   failures.append(str(error))
   continue
  if expected not in check(mutated):
   failures.append("mutation escaped " + expected)
 return failures


def verify_host_test_anchors(source_dir):
 anchors = (
  ("configuration.rs", "background_plist_requires_always_usage_keys"),
  ("configuration.rs", "legacy_always_key_requirement_tracks_deployment_target"),
  ("conversion.rs", "background_updates_require_always_and_report_downgrades"),
 )
 failures = []
 for filename, name in anchors:
  source = (source_dir / filename).read_text(encoding="utf-8")
  pattern = r"#\[test\]\s+fn\s+" + re.escape(name) + r"\s*\("
  if re.search(pattern, source) is None:
   failures.append("missing registered host test " + name)
   continue
  mutated = re.sub(r"\b" + re.escape(name) + r"\b", "removed_host_test", source, count=1)
  if re.search(pattern, mutated) is not None:
   failures.append("host test deletion mutation escaped " + name)
 return failures


def main():
 path = Path(sys.argv[1])
 source = path.read_text(encoding="utf-8")
 failures = check(source) + verify_negative_fixtures(source) + verify_host_test_anchors(path.parent)
 if failures:
  for failure in failures:
   print("FAIL: " + failure, file=sys.stderr)
  return 1
 print("location source and negative regressions passed")
 return 0


if __name__ == "__main__":
 raise SystemExit(main())
