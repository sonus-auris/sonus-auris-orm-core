#![allow(clippy::needless_return)]

const PRINCIPALS: [u8; 2] = [0, 1];
const CHUNK_INDICES: [u8; 3] = [0, 1, 2];

fn append_allowed(owner: u8, writer: u8, next_chunk: u8, requested_chunk: u8) -> bool {
    return owner == writer && requested_chunk == next_chunk;
}

fn validate_state(
    owner: u8,
    writer: u8,
    next_chunk: u8,
    requested_chunk: u8,
) -> Result<(), &'static str> {
    let allowed = append_allowed(owner, writer, next_chunk, requested_chunk);

    if allowed && owner != writer {
        return Err("cross-owner recording append admitted");
    }

    if allowed && requested_chunk != next_chunk {
        return Err("out-of-order chunk append admitted");
    }

    return Ok(());
}

fn run_model() -> Result<usize, &'static str> {
    let mut checked = 0usize;

    for owner in PRINCIPALS {
        for writer in PRINCIPALS {
            for next_chunk in CHUNK_INDICES {
                for requested_chunk in CHUNK_INDICES {
                    validate_state(owner, writer, next_chunk, requested_chunk)?;
                    checked += 1;
                }
            }
        }
    }

    return Ok(checked);
}

fn main() {
    match run_model() {
        Ok(checked) => {
            println!("recording append model: {checked} states");
            return;
        }
        Err(message) => {
            eprintln!("recording append model failed: {message}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{append_allowed, run_model};

    #[test]
    fn checks_all_thirty_six_states() {
        let checked = run_model().expect("bounded model must satisfy its invariants");
        assert_eq!(checked, 36);
        return;
    }

    #[test]
    fn append_requires_same_owner_and_exact_next_chunk() {
        assert!(append_allowed(0, 0, 0, 0));
        assert!(append_allowed(1, 1, 2, 2));
        assert!(!append_allowed(0, 1, 0, 0));
        assert!(!append_allowed(1, 0, 2, 2));
        assert!(!append_allowed(0, 0, 1, 0));
        assert!(!append_allowed(1, 1, 1, 2));
        return;
    }
}
