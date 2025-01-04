#[cfg(test)]
mod tests {
    #[ignore]
    #[tokio::test]
    async fn it_works_2() {
        let path = "/home/stan/Downloads/failedmaps/FAILED/badmaps/err13/Mission22.scx";

        let chk = bwmpq::get_chk_from_mpq_filename(path);

        match chk {
            Ok(chk) => {
                let parsed = bwmap::ParsedChk::from_bytes(&chk);
                let scenario_index = parsed
                    .sprp
                    .as_ref()
                    .unwrap()
                    .scenario_name_string_number
                    .clone() as usize;

                let scenario = parsed.get_string(scenario_index).unwrap();

                println!("{:96}: OK scenario: {:?}", path, scenario);
                // tokio::fs::rename(
                //     entry.path(),
                //     format!(
                //         "/home/stan/Downloads/failedmaps/ok/{}",
                //         entry.file_name().to_string_lossy()
                //     ),
                // )
                // .await
                // .unwrap();

                return;
            }
            Err(err) => println!("{:96}: ERR:{}", path, err),
        }
    }

    #[ignore]
    #[tokio::test]
    async fn it_works() {
        for entry in walkdir::WalkDir::new("/home/stan/Downloads/failedmaps/FAILED").into_iter() {
            let Ok(entry) = entry else {
                continue;
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let chk = bwmpq::get_chk_from_mpq_filename(entry.path());

            match chk {
                Ok(chk) => {
                    let parsed = bwmap::ParsedChk::from_bytes(&chk);
                    let scenario_index = parsed
                        .sprp
                        .as_ref()
                        .unwrap()
                        .scenario_name_string_number
                        .clone() as usize;

                    let scenario = parsed.get_string(scenario_index).unwrap();

                    println!("{:96}: OK scenario: {:?}", entry.path().display(), scenario);
                    // tokio::fs::rename(
                    //     entry.path(),
                    //     format!(
                    //         "/home/stan/Downloads/failedmaps/ok/{}",
                    //         entry.file_name().to_string_lossy()
                    //     ),
                    // )
                    // .await
                    // .unwrap();

                    continue;
                }
                Err(err) => println!("{:96}: ERR:{}", entry.path().display(), err),
            }
        }
    }
}
