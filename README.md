<div align="center">
<img src=".github/assets/banner.png"></img>

[![Build Status]][travis]

[Build Status]: https://github.com/pipebase/pipebase/actions/workflows/ci.yml/badge.svg
[travis]: https://github.com/pipebase/pipebase/actions?branch%3Amain
</div>
<br />

`pipebase` is a data integration framework, provides:

* *manifest*, specification of pipe / custom data object in YAML format
* *cli tool*, build data integration app with manifest
* *plugins*, customized pipes using third party SDK

[Quick Start] | [Examples] | [Templates]

## Overview
`pipebase` is composed of three main components
* **build**: [`pipegen`], [`cargo-pipe`], [`schema`]
* **runtime**: [`pipebase`]
* **plugins**: [`pipeware`]

[`cargo-pipe`]: https://github.com/pipebase/pipebase/tree/main/cargo-pipe
[`pipebase`]: https://github.com/pipebase/pipebase/tree/main/pipebase
[`pipegen`]: https://github.com/pipebase/pipebase/tree/main/pipegen
[`pipeware`]: https://github.com/pipebase/pipebase/tree/main/pipeware
[`examples`]: https://github.com/pipebase/pipebase/tree/main/examples
[Examples]: https://github.com/pipebase/pipebase/tree/main/examples
[`schema`]: https://github.com/pipebase/schema
[Templates]: https://github.com/pipebase/template
[Quick Start]: https://github.com/pipebase/pipebase/blob/main/cargo-pipe/README.md
