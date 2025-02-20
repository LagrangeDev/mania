default:
    @just --list

info:
    @echo JUST PATH: {{ just_executable() }}
    @echo GIT PATH: {{ require('git') }}
    @echo CARGO PATH: {{ require('cargo') }}
    @echo GREP PATH: {{ require('grep') }}
    @echo XARGS PATH: {{ require('xargs') }}
    @echo TYPOS PATH: {{ require('typos') }}
    @echo DENO PATH: {{ require('deno') }}
    @echo TAPLO PATH: {{ require('taplo') }}
    @echo SHFMT PATH: {{ require('shfmt') }}

check: info typoCheck fmtCheck clippyCheck buildCheck docCheck testCheck

typoCheck:
    typos -c ./typos.toml

fmtCheck: rustFmtCheck justFmtCheck mdFmtCheck tomlFmtCheck ymlFmtCheck shFmtCheck

fmt: rustFmt justFmt mdFmt tomlFmt ymlFmt shFmt

buildCheck:
    cargo build --locked

clippyCheck:
    cargo clippy --locked --all-targets -- --deny warnings

docCheck:
    cargo doc --no-deps --locked

testCheck:
    cargo test --locked

rustFmtCheck:
    cargo fmt --check

rustFmt:
    cargo fmt

justFmtCheck:
    just --unstable --fmt --check

justFmt:
    just --unstable --fmt

mdFmtCheck:
    git ls-files | grep -E '^.*\.(md|MD)$' | xargs deno fmt --check --ext md

mdFmt:
    git ls-files | grep -E '^.*\.(md|MD)$' | xargs deno fmt --ext md

ymlFmtCheck:
    git ls-files | grep -E '^.*\.yml$' | xargs deno fmt --check --ext yml

ymlFmt:
    git ls-files | grep -E '^.*\.yml$' | xargs deno fmt --ext yml

tomlFmtCheck:
    git ls-files | grep -E '^.*\.toml$' | xargs taplo format --check

tomlFmt:
    git ls-files | grep -E '^.*\.toml$' | xargs taplo format

shFmt:
    git ls-files | grep -E '^scripts/.*$' | xargs shfmt -p -s -i 2 -ci -sr -kp -fn -w

shFmtCheck:
    git ls-files | grep -E '^scripts/.*$' | xargs shfmt -p -s -i 2 -ci -sr -kp -fn -d

alias c := check
alias i := info
alias f := fmt
