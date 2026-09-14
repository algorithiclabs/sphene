# Security policy

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability.

Report it privately through GitHub's **Report a vulnerability** button on the repository's Security tab. If private vulnerability reporting is unavailable, email **domainique@algorithic.com** before disclosing details publicly.

Include:

- affected version or commit;
- operating system and ImageMagick/libheif/libavif versions, if relevant;
- minimal reproduction steps or proof of concept;
- expected and observed behavior;
- impact assessment and suggested mitigation, if known.

Allow maintainers reasonable time to investigate and coordinate disclosure. Do not include secrets or personal data in the report.

## Scope

Security reports include native image decoding, ImageMagick fallback argument forwarding, resource-limit bypasses, unsafe file or delegate access, and vulnerabilities in the shipped container configuration.
