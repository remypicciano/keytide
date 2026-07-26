# Security policy

## Supported versions

Security fixes currently target the latest `1.x` release and the default branch. Version 1 receives security fixes through July 13, 2027; severe issues may require upgrading to the newest compatible release.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private vulnerability reporting page:

<https://github.com/remypicciano/keytide/security/advisories/new>

Include the affected commit or version, operating system and architecture, a minimal reproduction using synthetic data, expected and observed behavior, and your assessment of impact. Do not send real `.cofferkey` files, passphrases, carrier files, confidential plaintext, or private containers.

You should receive an initial acknowledgement within five business days. Resolution timing depends on reproducibility, severity, and the need to preserve container compatibility. Please allow a reasonable remediation window before public disclosure.

The repository also runs weekly and pull-request scans for committed secrets, RustSec advisories, dependency licenses and sources, and risky dependency changes. Passing automation is not a substitute for independent review.

## Security boundary

KeyTide protects file contents and authenticated filename metadata while the container and its matching key remain separated. It does not protect secrets from malware or an administrator already controlling the user session, a compromised build or operating system, screen capture, active process-memory inspection, or disclosure of both the container and matching key.

No project representative will ask for an unlock key, passphrase, carrier file, or confidential plaintext as part of support or vulnerability triage.

The complete project assumptions, out-of-scope threats, residual risks, and v2 review questions are documented in the [threat model](docs/threat-model.md).
