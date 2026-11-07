#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// A rewrite is ready or already pasted.
    Finish,
    /// The shortcut run ended without replacing the selection.
    End,
}

const RATE: u32 = 44_100;

pub fn play(tone: Tone) {
    std::thread::spawn(move || {
        if let Err(error) = play_blocking(tone) {
            eprintln!("tone was not played: {error}");
        }
    });
}

pub fn render(tone: Tone) -> Vec<f32> {
    match tone {
        // Rising pair: the model finished a rewrite.
        Tone::Finish => notes(&[(659.25, 0.09), (0.0, 0.03), (987.77, 0.16)]),
        // Falling pair: the run ended and the original stayed.
        Tone::End => notes(&[(523.25, 0.08), (0.0, 0.025), (349.23, 0.15)]),
    }
}

fn notes(steps: &[(f64, f64)]) -> Vec<f32> {
    let mut out = Vec::new();
    for &(frequency, seconds) in steps {
        let count = (seconds * f64::from(RATE)).round() as usize;
        for index in 0..count {
            if frequency == 0.0 {
                out.push(0.0);
                continue;
            }
            let time = index as f64 / f64::from(RATE);
            let sample = (2.0 * std::f64::consts::PI * frequency * time).sin();
            out.push((sample * envelope(index, count) * 0.2) as f32);
        }
    }
    out
}

fn envelope(index: usize, count: usize) -> f64 {
    let attack = (RATE / 80) as usize;
    let release = (RATE / 28) as usize;
    let fade_in = if index < attack {
        index as f64 / attack.max(1) as f64
    } else {
        1.0
    };
    let remaining = count.saturating_sub(index);
    let fade_out = if remaining < release {
        remaining as f64 / release.max(1) as f64
    } else {
        1.0
    };
    fade_in * fade_out
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn play_blocking(tone: Tone) -> Result<(), String> {
    let samples = render(tone);
    let (_stream, handle) =
        rodio::OutputStream::try_default().map_err(|error| error.to_string())?;
    let sink = rodio::Sink::try_new(&handle).map_err(|error| error.to_string())?;
    sink.append(rodio::buffer::SamplesBuffer::new(1, RATE, samples));
    sink.sleep_until_end();
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn play_blocking(tone: Tone) -> Result<(), String> {
    let _ = render(tone);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finish_and_end_are_different_quiet_tones() {
        let finish = render(Tone::Finish);
        let end = render(Tone::End);
        assert!(finish.len() > RATE as usize / 5);
        assert!(end.len() > RATE as usize / 5);
        assert_ne!(finish, end);
        for sample in finish.iter().chain(end.iter()) {
            assert!(sample.is_finite());
            assert!(sample.abs() <= 0.21);
        }
        assert!(finish.iter().any(|sample| sample.abs() > 0.05));
        assert!(end.iter().any(|sample| sample.abs() > 0.05));
    }
}
