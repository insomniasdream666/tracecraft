pub struct ComponentRecommendation {
    pub name: String,
    pub description: String,
    pub score: f32,
}

pub fn keyword_recommend(context: &str) -> Vec<ComponentRecommendation> {
    let lower = context.to_lowercase();
    let keywords: Vec<(&str, &str, &str)> = vec![
        ("resistor", "Resistor", "Passive component for current limiting and voltage division"),
        ("capacitor", "Capacitor", "Passive component for filtering, decoupling, and energy storage"),
        ("inductor", "Inductor", "Passive component for energy storage in magnetic fields"),
        ("diode", "Diode", "Semiconductor for rectification and protection"),
        ("led", "LED", "Light-emitting diode for indication"),
        ("transistor", "Transistor", "Semiconductor for switching and amplification"),
        ("mosfet", "MOSFET", "Field-effect transistor for power switching"),
        ("opamp", "Op-Amp", "Operational amplifier for analog signal processing"),
        ("crystal", "Crystal Oscillator", "Precision frequency reference"),
        ("connector", "Connector", "Board-to-board or wire-to-board connector"),
        ("header", "Pin Header", "Male pin header for prototyping"),
        ("regulator", "Voltage Regulator", "Fixed or adjustable voltage regulation"),
        ("ldo", "LDO Regulator", "Low dropout voltage regulator"),
        ("dc dc", "DC-DC Converter", "Switching voltage regulator module"),
        ("microcontroller", "Microcontroller", "Programmable MCU with GPIO, ADC, etc"),
        ("mcu", "Microcontroller", "Programmable MCU with GPIO, ADC, etc"),
        ("fpga", "FPGA", "Field-programmable gate array"),
        ("sensor", "Sensor", "Environmental or position sensor"),
        ("temperature", "Temperature Sensor", "Measures ambient or contact temperature"),
        ("i2c", "I2C Device", "I2C-compatible peripheral"),
        ("spi", "SPI Device", "SPI-compatible peripheral"),
        ("uart", "UART Interface", "Serial communication interface"),
        ("usb", "USB Controller", "USB interface controller"),
        ("wifi", "WiFi Module", "Wireless connectivity module"),
        ("ble", "BLE Module", "Bluetooth low energy module"),
        ("rf", "RF Module", "Radio frequency transceiver module"),
        ("buzzer", "Buzzer", "Audio indicator"),
        ("relay", "Relay", "Electromechanical switch"),
        ("fuse", "Fuse", "Overcurrent protection device"),
        ("ferrite", "Ferrite Bead", "EMI suppression component"),
        ("tv", "TVS Diode", "Transient voltage suppression diode"),
    ];

    let mut results = Vec::new();
    for (keyword, name, desc) in keywords {
        if lower.contains(keyword) {
            results.push(ComponentRecommendation {
                name: name.to_string(),
                description: desc.to_string(),
                score: 0.5,
            });
            if results.len() >= 5 { break; }
        }
    }
    results
}
