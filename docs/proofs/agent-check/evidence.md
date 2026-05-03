# Agent Check Proof Evidence

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: codex/agent-check-proof

## Requirement

Proof-relevant PRs must include committed evidence in the form of a gameplay transcript, screenshot, or gif. CI must fail when that proof is absent.

## Red Check Before Evidence

Command:

```sh
rtk just agent-check
```

Observed result before this proof bundle existed:

```text
agent-check FAILED: proof-relevant changes require a changed artifact under docs/proofs/<proof-id>/.
Accepted evidence forms: gameplay transcript (.md or .txt), screenshot (.png/.jpg/.jpeg), or gif (.gif).
agent-check FAILED: proof-relevant changes require docs/proofs/<proof-id>/judge.md.
The judge file must include 'Verdict: sufficient' and 'Technical debt: clear'.
```

## Gameplay Transcript

Command:

```sh
rtk bash -lc 'cd parish && cargo run -p parish -- --script testing/fixtures/test_speed_assertions.txt'
```

Observed result:

```json
{"command":"/status","result":"system_command","response":"Location: Kilteevan Village | Morning | Spring","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["Location: Kilteevan Village | Morning | Spring","An older man heads off down the road.","An older woman with sharp eyes and herb-stained fingers heads off down the road.","A young woman heads off down the road.","A lean, red-haired young man with hard eyes heads off down the road."]}
{"command":"/speed","result":"system_command","response":"Speed: Normal","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["Speed: Normal"]}
{"command":"/speed slow","result":"system_command","response":"The parish slows to a gentle amble.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["The parish slows to a gentle amble."]}
{"command":"/speed fast","result":"system_command","response":"The parish quickens its step.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["The parish quickens its step."]}
{"command":"/speed normal","result":"system_command","response":"The parish settles into its natural stride.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["The parish settles into its natural stride."]}
{"command":"/speed bogus","result":"system_command","response":"Unknown speed 'bogus'. Try: slow, normal, fast, fastest, ludicrous.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["Unknown speed 'bogus'. Try: slow, normal, fast, fastest, ludicrous."]}
{"command":"/speed slow","result":"system_command","response":"The parish slows to a gentle amble.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["The parish slows to a gentle amble."]}
{"command":"/speed normal","result":"system_command","response":"The parish settles into its natural stride.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["The parish settles into its natural stride."]}
{"command":"/quit","result":"quit","location":"Kilteevan Village","time":"Morning","season":"Spring"}
```

## Local Walkthrough Smoke

Command:

```sh
rtk just game-test
```

Observed result:

```text
Finished dev profile, then ran target/debug/parish --script testing/fixtures/test_walkthrough.txt.
The script emitted JSON results for look, movement, status, map, and help commands and exited 0.
```
