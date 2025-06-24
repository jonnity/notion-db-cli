# notion-db-cli

A DB-focused CLI for Notion.so.
Helping to add many items to DB from CSV.

## Commands

* `db-list`: Show the list of databases
* `db-view`: Show the structure of the database matching the id specified by the argument
* `db-add`: Add the item to the database specified with id
* `db-query`: Query a database (Filters are not supported yet). In the other words, all items in the DB will be displayed.

If you want to get CSV to be used as a tmplate, do the following (`target DB id` can be confirmed with `db-list` or Notion Web UI).

```
notion-db-cli db-view {target DB id} --file template.csv 
```

Then, you can edit the `template.csv` to modify the first row (for example) and add rows.
Finally, if you execute the below command, the items described in csv will be added to the DB.

```
notion-db-cli db-add {target DB id} --file template.csv 
```

## How to use

1. Download binaries from [release](https://github.com/jonnity/notion-db-cli/releases) for your environment.
2. Create integration in ["Integrations" page of Notion](https://www.notion.so/profile/integrations) to generate "Internal Integration Secret".
3. Open the DB you want to handle in this CLI and connect the integration from the "Connections" menu.
4. Set the envionment variable `NOTION_CLI_RS_TOKEN` to "Internal Integration Secret" (or execute the binary with the `--token` option).

## Contributing

Interested in contributing? See [CONTRIBUTING.md](./CONTRIBUTING.md) for instructions.

In addition, if you use devcontainer to develop, you can set the token with `.devcontainer/.env` such as `NOTION_CLI_RS_TOKEN=ntn_xxxxxxxxxxxxxxxxxx`.
