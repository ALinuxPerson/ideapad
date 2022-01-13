# `ideapad`
A Rust utility library for some Lenovo IdeaPad specific functionality.

# A Fair Warning
This crate calls raw ACPI methods, which on the best case scenario when called on unsupported systems results in  
a `AE_NOT_FOUND` (acpi method not found) error, on the worst case you'll call an existing method, which might do
anything to your system.

This crate tries to safeguard against this by providing a profiles feature (see below for more details), which enforces
different methods for different models, however you could easily circumvent this by providing an arbitrary profile.

All in all, don't use this crate on unsupported systems.

# Supported Models

This crate has been tested on the Ideapad 15ILL05, although theoretically it should also work on the Ideapad AMD 
models.

| **Model**       | **Product Names**              |
|-----------------|--------------------------------|
| Ideapad 15ILL05 | 81YK                           |
| Ideapad AMD     | 81YQ (15ARE05), 81YM (14ARE05) |

# Dependencies
 * [`acpi_call`](https://github.com/mkottman/acpi_call): For calling ACPI methods.

# Features
## Battery Conservation
Battery conservation (mode) is a feature that allows you to save battery life by limiting the battery percentage to 60%.

```rust
ideapad::initialize()?;

if ideapad::battery_conservation::enabled()? {
    println!("Battery conservation mode is enabled");
} else if ideapad::battery_conservation::disabled()? {
    println!("Battery conservation mode is disabled");
} else {
    panic!("what");
}

battery_conservation::enable()?;
println!("Battery conservation mode should be enabled now...");

if battery_conservation::enabled()? {
    println!("...and it is.")
} else {
    panic!("...but it isn't!");
}

battery_conservation::disable()?;
println!("Battery conservation mode should be disabled now...");

if battery_conservation::disabled()? {
    println!("...and it is.")
} else {
    panic!("...but it isn't!");
}
```

This mode conflicts with rapid charging, because once the battery percentage is 60%, rapid charging will still try to
charge the battery more, but it can't, unnecessarily straining the battery.

There are various ways to mitigate this problem, provided by this crate:

 * `Ignore`: Ignore this problem entirely.
 * `Switch`: Switch off rapid charging, then enable battery conservation mode.
 * `Error`: Return an error to the caller.

For more information see `ideapad::Handler`.

```rust
ideapad::initialize()?;
ideapad::rapid_charge::enable()?;
ideapad::battery_conservation::enable()?; // the default handler is to switch
assert!(ideapad::rapid_charge::disabled()?);

ideapad::rapid_charge::enable()?;
ideapad::battery_conservation::enable_unchecked()?; // another handler is to ignore the problem entirely
assert!(rapid_charge::enabled()?);

let error = ideapad::battery_conservation::enable_strict().unwrap_err(); // another handler is to error
assert!(matches!(error, ideapad::battery_conservation_mode::Error::RapidChargeEnabled));

// you can arbitrarily choose which handler to use
ideapad::rapid_charge::enable_with_handler(ideapad::Handler::Ignore)?;
ideapad::rapid_charge::enable_with_handler(ideapad::Handler::Switch)?;
ideapad::rapid_charge::enable_with_handler(ideapad::Handler::Error)?;
```

## Rapid Charging
Rapid charging is a feature that allows you to charge your laptop faster. *I have no idea how this works.*

```rust
ideapad::initialize()?;

if ideapad::rapid_charge::enabled()? {
    println!("Rapid charge is enabled");
} else if ideapad::rapid_charge::disabled()? {
    println!("Rapid charge is disabled");
} else {
    panic!("what");
}

rapid_charge::enable()?;
println!("Rapid charge should be enabled now...");

if rapid_charge::enabled()? {
    println!("...and it is.")
} else {
    panic!("...but it isn't!");
}

rapid_charge::disable()?;
println!("Rapid charge should be disabled now...");

if battery_conservation::disabled()? {
    println!("...and it is.")
} else {
    panic!("...but it isn't!");
}
```

This mode conflicts with battery conservation. See the **Battery Conservation** section above for more information.

```rust
ideapad::initialize()?;
ideapad::battery_conservation::enable()?;
ideapad::rapid_charge::enable()?; // the default handler is to switch
assert!(ideapad::battery_conservation::disabled()?);

ideapad::battery_conservatio::enable()?;
ideapad::rapid_charge::enable_unchecked()?; // another handler is to ignore the problem entirely
assert!(battery_conservation::enabled()?);

let error = ideapad::rapid_charge::enable_strict().unwrap_err(); // another handler is to error
assert!(matches!(error, ideapad::rapid_charge::Error::BatteryConservationEnabled));

// you can arbitrarily choose which handler to use
ideapad::rapid_charge::enable_with_handler(ideapad::Handler::Ignore)?;
ideapad::rapid_charge::enable_with_handler(ideapad::Handler::Switch)?;
ideapad::rapid_charge::enable_with_handler(ideapad::Handler::Error)?;
```

## System Performance Mode
System performance mode are a variety of presets that can be set to improve the performance of your laptop.

 * **Extreme Performance**: As the name suggests, this mode will set the system to the highest performance possible. For **gamers**, I guess.
 * **Intelligent Cooling**: This mode throttles the CPU and lowers the fan noise.
 * **Battery Saving**: This mode will set the system to the lowest possible performance.

## Profiles
Note that this feature is not provided by Lenovo and is provided by this crate.

Profiles store ACPI methods and values that are used to set the system to a specific state. There are built in ones,
see **Supported Models** above, but you can also create your own (if you know what you're doing!).

# Supported Operating Systems
Currently, `ideapad` is only supported on Linux systems due to its aforementioned dependency on the 
[`acpi_call`](https://github.com/mkottman/acpi_call) kernel module.

After a few minutes of research, it seems like Windows does not support calling arbitrary ACPI methods in userspace. In
order to get around this, it seems a driver is needed to call the ACPI methods.

# Application
If you want to see this library being used in an application, see the 
[`tuxvantage`](https://github.com/ALinuxPerson/tuxvantage) project.