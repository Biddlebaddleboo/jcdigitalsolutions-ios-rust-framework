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


def impl_body(source, marker):
 start = source.find(marker)
 if start < 0:
  raise ValueError("missing impl " + marker)
 opening = source.find("{", start + len(marker))
 if opening < 0:
  raise ValueError("missing body for impl " + marker)
 depth = 0
 for index in range(opening, len(source)):
  if source[index] == "{":
   depth += 1
  elif source[index] == "}":
   depth -= 1
   if depth == 0:
    return source[start:index + 1]
 raise ValueError("unterminated impl " + marker)


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


def replace_impl(source, marker, old, new):
 start = source.find(marker)
 if start < 0:
  raise ValueError("missing impl " + marker)
 body = impl_body(source, marker)
 if old not in body:
  raise ValueError("negative fixture needle absent in impl " + marker)
 return source[:start] + body.replace(old, new, 1) + source[start + len(body):]


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
 for name in ("start_authorization_request", "start_always_authorization_request",
              "start_temporary_full_accuracy_authorization_request",
              "request_temporary_full_accuracy_authorization", "current_accuracy_authorization",
              "accuracy_authorization", "main_bundle_has_temporary_full_accuracy_usage_description",
              "start_current",
              "start_background_updates", "start_updates", "start_significant_change_updates",
              "open_geofence_monitor",
              "start_location_updates",
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
  ("requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(",
   "start_temporary_full_accuracy_authorization_request"),
  ("requestLocation()", "start_current"),
  ("startUpdatingLocation()", "start_location_updates"),
  ("startMonitoringSignificantLocationChanges()", "start_location_updates"),
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

 significant_method = functions["start_significant_change_updates"]
 require("LocationUpdateService::SignificantChange" in significant_method and
         "UpdateAuthorizationRequirement::Foreground" in significant_method and
         "None" in significant_method,
         "significant-change session is foreground-authorized and opt-in")
 require("requestWhenInUseAuthorization" not in significant_method and
         "requestAlwaysAuthorization" not in significant_method and
         "setAllowsBackgroundLocationUpdates" not in significant_method,
         "significant-change session does not prompt or enable standard background updates")

 geofence_open = functions["open_geofence_monitor"]
 runtime_guard = "objc2::available!(ios = 17.0, ..)"
 authorization_preflight = "geofence_authorization_preflight(authorization_from_native(native_authorization_status()))?"
 usage_config_preflight = "if !main_bundle_has_always_usage_descriptions()"
 prepare_monitor = "crate::geofence_platform::prepare_monitor_open(name, self.marker)"
 require(runtime_guard in geofence_open and authorization_preflight in geofence_open and
         usage_config_preflight in geofence_open and prepare_monitor in geofence_open and
         geofence_open.index(runtime_guard) < geofence_open.index(prepare_monitor) and
         geofence_open.index(authorization_preflight) < geofence_open.index(prepare_monitor) and
         geofence_open.index(usage_config_preflight) < geofence_open.index(prepare_monitor),
         "geofence iOS 17, Always authorization, and usage configuration preflight")
 require("requestWhenInUseAuthorization" not in geofence_open and
         "requestAlwaysAuthorization" not in geofence_open,
         "geofence preflight never prompts")

 accuracy_query = functions["accuracy_authorization"]
 accuracy_reader = functions["current_accuracy_authorization"]
 require("current_accuracy_authorization()" in accuracy_query and
         "requestTemporaryFullAccuracyAuthorization" not in accuracy_query and
         "requestWhenInUseAuthorization" not in accuracy_query and
         "requestAlwaysAuthorization" not in accuracy_query,
         "B87 accuracy snapshot stays non-prompting")
 require("objc2::available!(ios = 14.0, ..)" in accuracy_reader and
         "manager.accuracyAuthorization()" in accuracy_reader and
         "Unknown(raw)" in accuracy_reader,
         "B87 accuracy getter availability and unknown preservation")

 temporary_method = functions["request_temporary_full_accuracy_authorization"]
 temporary_start = functions["start_temporary_full_accuracy_authorization_request"]
 temporary_config = functions["main_bundle_has_temporary_full_accuracy_usage_description"]
 require("objc2::available!(ios = 14.0, ..)" in temporary_method and
         "is_valid_temporary_accuracy_purpose_key(purpose_key)" in temporary_method,
         "temporary accuracy request validates iOS floor and purpose-key syntax")
 require("objc2::available!(ios = 14.0, ..)" in temporary_start and
         "authorization_from_native(native_authorization_status()).allows_foreground()" in temporary_start and
         "current_accuracy_authorization()" in temporary_start and
         "main_bundle_has_temporary_full_accuracy_usage_description(purpose_key)" in temporary_start,
         "temporary accuracy request requires iOS 14, existing authorization, and declared purpose")
 require("requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(" in temporary_start and
         "RcBlock::new" in temporary_start and
         "requestTemporaryFullAccuracyAuthorization" not in accuracy_query,
         "temporary accuracy request is explicit and callback-scoped")
 require('NSLocationTemporaryUsageDescriptionDictionary' in temporary_config and
         "downcast::<NSDictionary>()" in temporary_config and
         "objectForKey(purpose_key)" in temporary_config and
         "description.length()" in temporary_config,
         "temporary accuracy purpose must match a nonempty Info.plist entry")

 updates = functions["start_location_updates"]
 mode_check = "if !main_bundle_has_location_background_mode()"
 background_setter = "manager.setAllowsBackgroundLocationUpdates(true)"
 try:
  background_branch = block_after(updates, "UpdateAuthorizationRequirement::Background,")
 except ValueError:
  background_branch = ""
 require(mode_check in background_branch, "background plist preflight branch")
 require("if !updates_permitted(authorization, requirement)" in updates,
         "session authorization check")
 require(updates.count("manager.startMonitoringSignificantLocationChanges()") == 1 and
         source.count("manager.startMonitoringSignificantLocationChanges()") == 1,
         "significant selector scope")
 try:
  significant_preflight = block_after(
   updates, "if matches!(service, LocationUpdateService::SignificantChange)")
 except ValueError:
  significant_preflight = ""
 support_query = "CLLocationManager::significantLocationChangeMonitoringAvailable()"
 support_check = "significant_change_preflight(supported, authorization)?"
 require(support_query in significant_preflight and support_check in significant_preflight and
         significant_preflight.find(support_query) < significant_preflight.find(support_check),
         "significant device support preflight")
 require("if let LocationUpdateService::Standard(request) = service" in updates and
         "LocationUpdateService::SignificantChange => unsafe" in updates,
         "significant-change service does not apply standard accuracy settings")
 try:
  updates_drop = impl_body(source, "impl Drop for IosLocationUpdates")
 except ValueError:
  updates_drop = ""
 require("LocationUpdateService::SignificantChange" in updates_drop and
         "self.manager.stopMonitoringSignificantLocationChanges()" in updates_drop,
         "significant-change session drop stops its matching native service")
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


def check_geofence_contract(contract_source, native_source):
 errors = []
 def require(condition, label):
  if not condition:
   errors.append(label)

 require("const GEOFENCE_EVENT_QUEUE_CAPACITY: usize = 16" in contract_source,
         "geofence event queue capacity")
 require("MonitorLease::acquire()" in native_source and
         "MonitorAlreadyOpen" in contract_source and "compare_exchange(false, true" in contract_source,
         "single live monitor lease rejects duplicate opens")
 require('DispatchQueue::new("com.jcdigitalsolutions.ios-location.geofence", None)' in native_source and
         "CLMonitorConfiguration::configWithMonitorName_queue_eventHandler" in native_source and
         "CLMonitor::requestMonitorWithConfiguration_completion" in native_source and
         "shared.queue.as_ref()" in native_source,
         "CLMonitor uses configured serial queue")
 require("fn with_monitor" in native_source and "self.queue.exec_sync(||" in native_source,
         "native monitor access is serialized on its configured queue")
 for method in ("add_or_replace", "remove", "list"):
  try:
   body = function_body(native_source, method)
  except ValueError:
   body = ""
  require("self.shared.with_monitor(" in body, "geofence operation queue scope " + method)
 try:
  add_replace = function_body(native_source, "add_or_replace")
 except ValueError:
  add_replace = ""
 require("monitor.addConditionForMonitoring_identifier" in add_replace and
         "removeConditionFromMonitoringWithIdentifier" not in add_replace,
         "same-identifier replacement directly calls CLMonitor add")
 require("CLCircularGeographicCondition::initWithCenter_radius" in native_source and
         "monitor.addConditionForMonitoring_identifier" in native_source and
         "monitor.removeConditionFromMonitoringWithIdentifier" in native_source,
         "geofence add replace and remove use CLMonitor condition API")
 require("available!(ios = 18.0, ..)" in native_source and
         "event.conditionUnsupported()" in native_source and
         "event.conditionLimitExceeded()" in native_source and
         "event.persistenceUnavailable()" in native_source and
         "event.authorizationDenied()" in native_source,
         "typed native issue flags are availability guarded")
 require("complete_with_deferred" in native_source and "push_deferred" in native_source and
         native_source.count("wake_off_monitor_queue(waker)") >= 2 and
         "DispatchQueue::global_queue" in native_source and "queue.exec_async" in native_source,
         "foreign callback wakes are deferred off monitor queue")
 try:
  monitor_drop = impl_body(native_source, "impl Drop for IosGeofenceMonitor {")
 except ValueError:
  monitor_drop = ""
 require("self.shared.events.detach()" in monitor_drop and
         "self.shared.close_on_queue()" in monitor_drop and
         "removeConditionFromMonitoringWithIdentifier" not in monitor_drop,
         "geofence conditions remain persisted on session drop")
 try:
  monitor_remove = function_body(native_source, "remove")
 except ValueError:
  monitor_remove = ""
 require("removeConditionFromMonitoringWithIdentifier" in monitor_remove,
         "geofence conditions have an explicit removal operation")
 try:
  open_poll = function_body(native_source, "poll")
  open_success = block_after(open_poll, "Poll::Ready(Ok(()))")
  open_failure = block_after(open_poll, "Poll::Ready(Err(error))")
 except ValueError:
  open_poll = ""
  open_success = ""
  open_failure = ""
 require("this.shared.take()" in open_success and "this.shared.take()" in open_failure and
         "shared.close_on_queue()" in open_failure and "shared.events.detach()" in open_failure,
         "open future transfers ownership and promptly releases terminal failures")
 prohibited = re.search(r"startMonitoringForRegion|stopMonitoringForRegion|CLCircularRegion|\bCLRegion\b", native_source)
 require(prohibited is None, "geofence uses modern CLMonitor API only")
 return errors


def verify_geofence_negative_fixtures(contract_source, native_source):
 failures = []
 mutations = (
  ("contract", contract_source, "const GEOFENCE_EVENT_QUEUE_CAPACITY: usize = 16", "const GEOFENCE_EVENT_QUEUE_CAPACITY: usize = 17", "geofence event queue capacity"),
  ("native", native_source, "CLCircularGeographicCondition::initWithCenter_radius", "CLCircularRegion::init", "geofence uses modern CLMonitor API only"),
 )
 for role, source, old, new, expected in mutations:
  if old not in source:
   failures.append("negative fixture needle absent: " + expected)
   continue
  changed = source.replace(old, new, 1)
  changed_contract = changed if role == "contract" else contract_source
  changed_native = changed if role == "native" else native_source
  if expected not in check_geofence_contract(changed_contract, changed_native):
   failures.append("mutation escaped " + expected)
 try:
  platform_source = (Path(__file__).parent / "src" / "platform.rs").read_text(encoding="utf-8")
  mutated = replace_function(
   platform_source,
   "open_geofence_monitor",
   "objc2::available!(ios = 17.0, ..)",
   "();",
  )
 except (ValueError, OSError) as error:
  failures.append("geofence permission negative fixture: " + str(error))
 else:
  if "geofence iOS 17, Always authorization, and usage configuration preflight" not in check(mutated):
   failures.append("geofence iOS 17 availability mutation escaped")
 try:
  mutated = replace_function(
   platform_source,
   "open_geofence_monitor",
   "geofence_authorization_preflight(authorization_from_native(native_authorization_status()))?;",
   "();",
  )
 except ValueError as error:
  failures.append("geofence permission negative fixture: " + str(error))
 else:
  if "geofence iOS 17, Always authorization, and usage configuration preflight" not in check(mutated):
   failures.append("geofence permission removal mutation escaped")
 try:
  mutated = replace_function(
   platform_source,
   "open_geofence_monitor",
   "if !main_bundle_has_always_usage_descriptions()",
   "if false",
  )
 except ValueError as error:
  failures.append("geofence configuration negative fixture: " + str(error))
 else:
  if "geofence iOS 17, Always authorization, and usage configuration preflight" not in check(mutated):
   failures.append("geofence usage-description mutation escaped")
 try:
  mutated = replace_impl(
   native_source,
   "impl Drop for IosGeofenceMonitor {",
   "self.shared.close_on_queue();",
   'self.shared.close_on_queue();\n        let _ = self.shared.with_monitor(|monitor| unsafe { monitor.removeConditionFromMonitoringWithIdentifier(&NSString::from_str("drop")) });',
  )
 except ValueError as error:
  failures.append("geofence persistence negative fixture: " + str(error))
 else:
  if "geofence conditions remain persisted on session drop" not in check_geofence_contract(contract_source, mutated):
   failures.append("geofence implicit-removal mutation escaped")
 try:
  mutated = replace_function(
   native_source,
   "add_or_replace",
   "monitor.addConditionForMonitoring_identifier(&condition, &identifier);",
   "monitor.removeConditionFromMonitoringWithIdentifier(&identifier);\n            monitor.addConditionForMonitoring_identifier(&condition, &identifier);",
  )
 except ValueError as error:
  failures.append("geofence replacement negative fixture: " + str(error))
 else:
  if "same-identifier replacement directly calls CLMonitor add" not in check_geofence_contract(contract_source, mutated):
   failures.append("geofence pre-removal replacement mutation escaped")
 try:
  mutated = replace_function(
   native_source,
   "poll",
   "this.shared.take()",
   "this.shared.as_ref().cloned()",
  )
 except ValueError as error:
  failures.append("geofence ownership negative fixture: " + str(error))
 else:
  if "open future transfers ownership and promptly releases terminal failures" not in check_geofence_contract(contract_source, mutated):
   failures.append("geofence lease-transfer mutation escaped")
 return failures


def verify_negative_fixtures(source):
 cases = (
  ("selector scope requestWhenInUseAuthorization()", "start_authorization_request", "manager.requestWhenInUseAuthorization()", "removed_request()"),
  ("selector scope requestAlwaysAuthorization()", "start_always_authorization_request", "manager.requestAlwaysAuthorization()", "removed_request()"),
  ("selector scope requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(", "start_temporary_full_accuracy_authorization_request", "requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(", "removed_request()"),
  ("selector scope requestLocation()", "start_current", "manager.requestLocation()", "removed_request()"),
  ("selector scope startUpdatingLocation()", "start_location_updates", "manager.startUpdatingLocation()", "removed_update()"),
  ("selector scope setAllowsBackgroundLocationUpdates(true)", "start_location_updates", "manager.setAllowsBackgroundLocationUpdates(true)", "removed_background_enable()"),
  ("Always request eligibility", "start_always_authorization_request", "if !should_request_always_authorization(authorization)", ""),
  ("Always config before request", "start_always_authorization_request", "if !main_bundle_has_always_usage_descriptions()", ""),
  ("Always usage-key mapping", "main_bundle_has_always_usage_descriptions", "NSLocationWhenInUseUsageDescription", "RemovedWhenInUseUsageKey"),
  ("Always usage-key mapping", "main_bundle_has_always_usage_descriptions", "NSLocationAlwaysAndWhenInUseUsageDescription", "RemovedUsageKey"),
  ("Always usage-key mapping", "main_bundle_has_always_usage_descriptions", "NSLocationAlwaysUsageDescription", "RemovedLegacyUsageKey"),
  ("temporary accuracy request validates iOS floor and purpose-key syntax", "request_temporary_full_accuracy_authorization", "is_valid_temporary_accuracy_purpose_key(purpose_key)", "true"),
  ("temporary accuracy request requires iOS 14, existing authorization, and declared purpose", "start_temporary_full_accuracy_authorization_request", "authorization_from_native(native_authorization_status()).allows_foreground()", "true"),
  ("temporary accuracy request requires iOS 14, existing authorization, and declared purpose", "start_temporary_full_accuracy_authorization_request", "main_bundle_has_temporary_full_accuracy_usage_description(purpose_key)", "true"),
  ("temporary accuracy purpose must match a nonempty Info.plist entry", "main_bundle_has_temporary_full_accuracy_usage_description", "NSLocationTemporaryUsageDescriptionDictionary", "RemovedTemporaryUsageKey"),
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
  ("significant selector scope", "start_location_updates", "manager.startMonitoringSignificantLocationChanges()", "removed_significant_start()"),
  ("significant device support preflight", "start_location_updates", "CLLocationManager::significantLocationChangeMonitoringAvailable()", "true"),
  ("significant device support preflight", "start_location_updates", "significant_change_preflight(supported, authorization)?", "Ok(())"),
  ("significant-change session is foreground-authorized and opt-in", "start_significant_change_updates", "UpdateAuthorizationRequirement::Foreground", "UpdateAuthorizationRequirement::Background"),
  ("significant-change session is foreground-authorized and opt-in", "start_significant_change_updates", "LocationUpdateService::SignificantChange", "LocationUpdateService::Standard(LocationRequest::new(AccuracyTarget::new(1.0).unwrap()))"),
 )
 failures = []
 wrong_scope_cases = (
  ("selector scope startUpdatingLocation()", "start_current", "let native_accuracy =", "unsafe { manager.startUpdatingLocation() };"),
  ("selector scope requestLocation()", "start_location_updates", "if !updates_permitted(authorization, requirement)", "unsafe { manager.requestLocation() };"),
  ("selector scope requestAlwaysAuthorization()", "start_authorization_request", "let status = native_authorization_status();", "unsafe { manager.requestAlwaysAuthorization() };"),
  ("selector scope requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(", "start_current", "let native_accuracy =", "unsafe { manager.requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(purpose_key, None) };"),
  ("significant selector scope", "start_updates", "start_location_updates(", "unsafe { manager.startMonitoringSignificantLocationChanges() };"),
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
 try:
  mutated = replace_impl(
   source,
   "impl Drop for IosLocationUpdates",
   "self.manager.stopMonitoringSignificantLocationChanges()",
   "self.manager.stopUpdatingLocation()",
  )
 except ValueError as error:
  failures.append(str(error))
 else:
  if "significant-change session drop stops its matching native service" not in check(mutated):
   failures.append("significant-change stop mutation escaped session-drop guard")
 return failures


def verify_host_test_anchors(source_dir):
 anchors = (
  ("configuration.rs", "background_plist_requires_always_usage_keys"),
  ("configuration.rs", "legacy_always_key_requirement_tracks_deployment_target"),
  ("configuration.rs", "temporary_accuracy_purpose_keys_reject_empty_and_nul"),
  ("conversion.rs", "background_updates_require_always_and_report_downgrades"),
  ("conversion.rs", "significant_change_preflight_requires_device_support_and_existing_authorization"),
  ("geofence.rs", "geofence_validates_each_negative_input"),
  ("geofence.rs", "monitor_name_rules_reject_empty_underscore_and_non_alphanumeric_values"),
  ("geofence.rs", "authorization_preflight_requires_already_granted_always_without_prompting"),
  ("geofence.rs", "native_state_mapping_preserves_known_states_and_unknown_values"),
  ("geofence.rs", "event_queue_preserves_order_and_reports_evicted_count_on_delivery"),
  ("geofence.rs", "event_wait_cancel_and_late_open_callback_are_detached_safely"),
  ("geofence.rs", "deferred_native_completion_wake_does_not_run_inline_and_monitor_lease_reopens"),
  ("geofence.rs", "callback_copy_failure_is_separate_from_eviction_count"),
  ("geofence.rs", "native_issue_flags_map_to_typed_authorization_support_and_persistence_issues"),
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
 try:
  contract_source = (path.parent / "geofence.rs").read_text(encoding="utf-8")
  native_source = (path.parent / "geofence_platform.rs").read_text(encoding="utf-8")
 except OSError as error:
  print("FAIL: unable to inspect geofence source: " + str(error), file=sys.stderr)
  return 1
 failures = (check(source) + verify_negative_fixtures(source) + verify_host_test_anchors(path.parent) +
             check_geofence_contract(contract_source, native_source) +
             verify_geofence_negative_fixtures(contract_source, native_source))
 if failures:
  for failure in failures:
   print("FAIL: " + failure, file=sys.stderr)
  return 1
 print("location source and negative regressions passed")
 return 0


if __name__ == "__main__":
 raise SystemExit(main())
