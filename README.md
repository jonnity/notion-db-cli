# notion-cli-rs

A CLI tool for Notion.so built with Rust.

## Setup

1. Create integration in ["Integrations" page of Notion](https://www.notion.so/profile/integrations) to generate "Internal Integration Secret".
2. Open the DB you want to handle in this CLI and connect the integration from the "Connections" menu.
3. Set the envionment variable `NOTION_CLI_RS_TOKEN` to "Internal Integration Secret".

(In the future, logins via Public Integration might also be supported.)
