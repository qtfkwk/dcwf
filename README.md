# About

* Fetch necessary HTML files from the DCWF ([DoD Cyber Workforce Framework])
* Save HTML files to the [`data`] directory (or use `-d`) to avoid downloading again
* Force fresh download by specifying a different directory or deleting the data directory
* Extract elements, roles, and KSATs from the downloaded HTML pages
* Transform to data structure (index, dedupe, interlink)
* Print as JSON

# Usage

```text
$ dcwf -h
Utility to fetch raw JSON data from the DoD Cyber Workforce Framework

Usage: dcwf [OPTIONS]

Options:
  -f <FORMAT>      Output format (json, json-pretty) [default: json]
  -d <PATH>        Data directory [default: data]
  -h, --help       Print help
  -V, --version    Print version
```

```text
$ dcwf -V
dcwf 0.1.1
```

# Examples

```text
$ dcwf >data.json
Read from "data/elements.html"
Read from "data/elements/it-cyberspace.html"
Read from "data/elements/cybersecurity.html"
Read from "data/elements/cyberspace-effects.html"
Read from "data/elements/intelligence-cyberspace.html"
Read from "data/elements/acquisition.html"
Read from "data/elements/leadership.html"
Read from "data/elements/legal-law-enforcement.html"
Read from "data/elements/training-and-education.html"
Read from "data/elements/software-engineering.html"
Read from "data/elements/ai-data.html"
```

*See the result in [`data.json`].*

Use any other tools you want to process the JSON (like [`jq`]...).

```text
$ dcwf -f json-pretty >pretty.json
Read from "data/elements.html"
Read from "data/elements/it-cyberspace.html"
Read from "data/elements/cybersecurity.html"
Read from "data/elements/cyberspace-effects.html"
Read from "data/elements/intelligence-cyberspace.html"
Read from "data/elements/acquisition.html"
Read from "data/elements/leadership.html"
Read from "data/elements/legal-law-enforcement.html"
Read from "data/elements/training-and-education.html"
Read from "data/elements/software-engineering.html"
Read from "data/elements/ai-data.html"
```

*See the result in [`pretty.json`].*

Delete the data directory and run again:

```text
$ rm -rf data
$ dcwf >data.json
Fetching "https://public.cyber.mil/wid/dcwf/workforce-elements"... saved to "data/elements.html"
Fetching "https://public.cyber.mil/wf-element-sub/it-cyberspace/"... saved to "data/elements/it-cyberspace.html"
Fetching "https://public.cyber.mil/wf-element-sub/cybersecurity/"... saved to "data/elements/cybersecurity.html"
Fetching "https://public.cyber.mil/wf-element-sub/cyberspace-effects/"... saved to "data/elements/cyberspace-effects.html"
Fetching "https://public.cyber.mil/wf-element-sub/intelligence-cyberspace/"... saved to "data/elements/intelligence-cyberspace.html"
Fetching "https://public.cyber.mil/wf-element-sub/acquisition/"... saved to "data/elements/acquisition.html"
Fetching "https://public.cyber.mil/wf-element-sub/leadership/"... saved to "data/elements/leadership.html"
Fetching "https://public.cyber.mil/wf-element-sub/legal-law-enforcement/"... saved to "data/elements/legal-law-enforcement.html"
Fetching "https://public.cyber.mil/wf-element-sub/training-and-education/"... saved to "data/elements/training-and-education.html"
Fetching "https://public.cyber.mil/wf-element-sub/software-engineering/"... saved to "data/elements/software-engineering.html"
Fetching "https://public.cyber.mil/wf-element-sub/ai-data/"... saved to "data/elements/ai-data.html"
```

*Note that this takes approximately 1 minute to download all files.*
  
[`data`]: data
[`data.json`]: data.json
[`jq`]: https://jqlang.github.io/jq/
[`pretty.json`]: pretty.json
[DoD Cyber Workforce Framework]: https://public.cyber.mil/cw/dcwf/

