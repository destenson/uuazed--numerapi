use std::collections::BTreeMap;

use clap::{command, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command};

#[cfg(feature = "async")]
use tokio;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    println!("Running in async mode");
    async_function().await;
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("Running in sync mode");
    // sync_function();
}

fn parse_args() -> (ArgMatches, Vec<(clap::Id, Value)>) {
    let matches = cli().get_matches();
    let values = Value::from_matches(&matches);
    println!("{values:#?}");
    (matches, values)
}

fn cli() -> Command {
    fn position_sensitive_flag(arg: Arg) -> Arg {
        // Flags don't track the position of each occurrence, so we need to emulate flags with
        // value-less options to get the same result
        arg.num_args(0)
            .value_parser(value_parser!(bool))
            .default_missing_value("true")
            .default_value("false")
    }

    command!()
        .group(ArgGroup::new("tests").multiple(true))
        .next_help_heading("TESTS")
        .args([
            position_sensitive_flag(Arg::new("empty"))
                .long("empty")
                .action(ArgAction::Append)
                .help("File is empty and is either a regular file or a directory")
                .group("tests"),
            Arg::new("name")
                .long("name")
                .action(ArgAction::Append)
                .help("Base of file name (the path with the leading directories removed) matches shell pattern pattern")
                .group("tests")
        ])
        .group(ArgGroup::new("operators").multiple(true))
        .next_help_heading("OPERATORS")
        .args([
            position_sensitive_flag(Arg::new("or"))
                .short('o')
                .long("or")
                .action(ArgAction::Append)
                .help("expr2 is not evaluate if exp1 is true")
                .group("operators"),
            position_sensitive_flag(Arg::new("and"))
                .short('a')
                .long("and")
                .action(ArgAction::Append)
                .help("Same as `expr1 expr1`")
                .group("operators"),
        ])
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Bool(bool),
    String(String),
}

impl Value {
    pub fn from_matches(matches: &ArgMatches) -> Vec<(clap::Id, Self)> {
        let mut values = BTreeMap::new();
        for id in matches.ids() {
            if matches.try_get_many::<clap::Id>(id.as_str()).is_ok() {
                // ignore groups
                continue;
            }
            let value_source = matches
                .value_source(id.as_str())
                .expect("id came from matches");
            if value_source != clap::parser::ValueSource::CommandLine {
                // Any other source just gets tacked on at the end (like default values)
                continue;
            }
            if Self::extract::<String>(matches, id, &mut values) {
                continue;
            }
            if Self::extract::<bool>(matches, id, &mut values) {
                continue;
            }
            unimplemented!("unknown type for {id}: {matches:?}");
        }
        values.into_values().collect::<Vec<_>>()
    }

    fn extract<T: Clone + Into<Value> + Send + Sync + 'static>(
        matches: &ArgMatches,
        id: &clap::Id,
        output: &mut BTreeMap<usize, (clap::Id, Self)>,
    ) -> bool {
        match matches.try_get_many::<T>(id.as_str()) {
            Ok(Some(values)) => {
                for (value, index) in values.zip(
                    matches
                        .indices_of(id.as_str())
                        .expect("id came from matches"),
                ) {
                    output.insert(index, (id.clone(), value.clone().into()));
                }
                true
            }
            Ok(None) => {
                unreachable!("`ids` only reports what is present")
            }
            Err(clap::parser::MatchesError::UnknownArgument { .. }) => {
                unreachable!("id came from matches")
            }
            Err(clap::parser::MatchesError::Downcast { .. }) => false,
            Err(_) => {
                unreachable!("id came from matches")
            }
        }
    }
}

impl From<String> for Value {
    fn from(other: String) -> Self {
        Self::String(other)
    }
}

impl From<bool> for Value {
    fn from(other: bool) -> Self {
        Self::Bool(other)
    }
}


/*
""" Access the numerai API via command line"""

import json
import datetime
import decimal

import click

import numerapi

try:
    from dotenv import load_dotenv
    load_dotenv()
except:
    try:
        import os
        locations = ['./.env', '../.env', '../../.env', os.path.expanduser('~/.env')].reverse()
        for location in locations:
            if os.path.exists(location):
                with open(location) as f:
                    for line in f:
                        # ignore comments and empty lines
                        if line.startswith('#') or not line.strip():
                            continue
                        if line.contains('='):
                            key, value = line.strip().split('=', 1)
                            os.environ[key] = value

    except:
        pass

napi = numerapi.NumerAPI()

class CommonJSONEncoder(json.JSONEncoder):
    """
    Common JSON Encoder
    json.dumps(jsonString, cls=CommonJSONEncoder)
    """
    def default(self, o):
        # Encode: Decimal
        if isinstance(o, decimal.Decimal):
            return str(o)
        # Encode: Date & Datetime
        if isinstance(o, (datetime.date, datetime.datetime)):
            return o.isoformat()

        return None


def prettify(stuff):
    """prettify json"""
    return json.dumps(stuff, cls=CommonJSONEncoder, indent=4)


@click.group()
def cli():
    """Wrapper around the Numerai API"""


@cli.command()
@click.option('--round_num',
              help='round you are interested in.defaults to the current round')
def list_datasets(round_num):
    """List of available data files"""
    click.echo(prettify(napi.list_datasets(round_num=round_num)))


@cli.command()
@click.option(
    '--round_num',
    help='round you are interested in.defaults to the current round')
@click.option(
    '--filename', help='file to be downloaded')
@click.option(
    '--dest_path',
    help='complate destination path, defaults to the name of the source file')
def download_dataset(round_num, filename="numerai_live_data.parquet",
                     dest_path=None):
    """Download specified file for the given round"""
    click.echo("WARNING to download the old data use `download-dataset-old`")
    click.echo(napi.download_dataset(
        round_num=round_num, filename=filename, dest_path=dest_path))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
def competitions(tournament=8):
    """Retrieves information about all competitions"""
    click.echo(prettify(napi.get_competitions(tournament=tournament)))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
def current_round(tournament=8):
    """Get number of the current active round."""
    click.echo(napi.get_current_round(tournament=tournament))


@cli.command()
@click.option('--limit', default=20,
              help='Number of items to return, defaults to 20')
@click.option('--offset', default=0,
              help='Number of items to skip, defaults to 0')
def leaderboard(limit=20, offset=0):
    """Get the leaderboard."""
    click.echo(prettify(napi.get_leaderboard(limit=limit, offset=offset)))


@cli.command()
@click.option('--tournament', type=int, default=None,
              help='filter by ID of the tournament, defaults to None')
@click.option('--round_num', type=int, default=None,
              help='filter by round number, defaults to None')
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def submission_filenames(round_num, tournament, model_id):
    """Get filenames of your submissions"""
    click.echo(prettify(
        napi.get_submission_filenames(tournament, round_num, model_id)))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
@click.option('--hours', default=12,
              help='timeframe to consider, defaults to 12')
def check_new_round(hours=12, tournament=8):
    """Check if a new round has started within the last `hours`."""
    click.echo(int(napi.check_new_round(hours=hours, tournament=tournament)))


@cli.command()
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def user(model_id):
    """Get all information about you! DEPRECATED - use account"""
    click.echo(prettify(napi.get_user(model_id)))


@cli.command()
def account():
    """Get all information about your account!"""
    click.echo(prettify(napi.get_account()))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
def models(tournament):
    """Get map of account models!"""
    click.echo(prettify(napi.get_models(tournament)))


@cli.command()
@click.argument("username")
def profile(username):
    """Fetch the public profile of a user."""
    click.echo(prettify(napi.public_user_profile(username)))


@cli.command()
@click.argument("username")
def daily_model_performances(username):
    """Fetch daily performance of a model."""
    click.echo(prettify(napi.daily_model_performances(username)))


@cli.command()
def transactions():
    """List all your deposits and withdrawals."""
    click.echo(prettify(napi.wallet_transactions()))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
@click.argument('path', type=click.Path(exists=True))
def submit(path, tournament, model_id):
    """Upload predictions from file."""
    click.echo(napi.upload_predictions(
        path, tournament, model_id))


@cli.command()
@click.argument("username")
def stake_get(username):
    """Get stake value of a user."""
    click.echo(napi.stake_get(username))


@cli.command()
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def stake_drain(model_id):
    """Completely remove your stake."""
    click.echo(napi.stake_drain(model_id))


@cli.command()
@click.argument("nmr")
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def stake_decrease(nmr, model_id):
    """Decrease your stake by `value` NMR."""
    click.echo(napi.stake_decrease(nmr, model_id))


@cli.command()
@click.argument("nmr")
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def stake_increase(nmr, model_id):
    """Increase your stake by `value` NMR."""
    click.echo(napi.stake_increase(nmr, model_id))


@cli.command()
def version():
    """Installed numerapi version."""
    print(numerapi.__version__)


*/

/*
""" Access the numerai API via command line"""

import json
import datetime
import decimal

import click

import numerapi

try:
    from dotenv import load_dotenv
    load_dotenv()
except:
    try:
        import os
        locations = ['./.env', '../.env', '../../.env', os.path.expanduser('~/.env')].reverse()
        for location in locations:
            if os.path.exists(location):
                with open(location) as f:
                    for line in f:
                        # ignore comments and empty lines
                        if line.startswith('#') or not line.strip():
                            continue
                        if line.contains('='):
                            key, value = line.strip().split('=', 1)
                            os.environ[key] = value

    except:
        pass

napi = numerapi.NumerAPI()

class CommonJSONEncoder(json.JSONEncoder):
    """
    Common JSON Encoder
    json.dumps(jsonString, cls=CommonJSONEncoder)
    """
    def default(self, o):
        # Encode: Decimal
        if isinstance(o, decimal.Decimal):
            return str(o)
        # Encode: Date & Datetime
        if isinstance(o, (datetime.date, datetime.datetime)):
            return o.isoformat()

        return None


def prettify(stuff):
    """prettify json"""
    return json.dumps(stuff, cls=CommonJSONEncoder, indent=4)


@click.group()
def cli():
    """Wrapper around the Numerai API"""


@cli.command()
@click.option('--round_num',
              help='round you are interested in.defaults to the current round')
def list_datasets(round_num):
    """List of available data files"""
    click.echo(prettify(napi.list_datasets(round_num=round_num)))


@cli.command()
@click.option(
    '--round_num',
    help='round you are interested in.defaults to the current round')
@click.option(
    '--filename', help='file to be downloaded')
@click.option(
    '--dest_path',
    help='complate destination path, defaults to the name of the source file')
def download_dataset(round_num, filename="numerai_live_data.parquet",
                     dest_path=None):
    """Download specified file for the given round"""
    click.echo("WARNING to download the old data use `download-dataset-old`")
    click.echo(napi.download_dataset(
        round_num=round_num, filename=filename, dest_path=dest_path))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
def competitions(tournament=8):
    """Retrieves information about all competitions"""
    click.echo(prettify(napi.get_competitions(tournament=tournament)))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
def current_round(tournament=8):
    """Get number of the current active round."""
    click.echo(napi.get_current_round(tournament=tournament))


@cli.command()
@click.option('--limit', default=20,
              help='Number of items to return, defaults to 20')
@click.option('--offset', default=0,
              help='Number of items to skip, defaults to 0')
def leaderboard(limit=20, offset=0):
    """Get the leaderboard."""
    click.echo(prettify(napi.get_leaderboard(limit=limit, offset=offset)))


@cli.command()
@click.option('--tournament', type=int, default=None,
              help='filter by ID of the tournament, defaults to None')
@click.option('--round_num', type=int, default=None,
              help='filter by round number, defaults to None')
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def submission_filenames(round_num, tournament, model_id):
    """Get filenames of your submissions"""
    click.echo(prettify(
        napi.get_submission_filenames(tournament, round_num, model_id)))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
@click.option('--hours', default=12,
              help='timeframe to consider, defaults to 12')
def check_new_round(hours=12, tournament=8):
    """Check if a new round has started within the last `hours`."""
    click.echo(int(napi.check_new_round(hours=hours, tournament=tournament)))


@cli.command()
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def user(model_id):
    """Get all information about you! DEPRECATED - use account"""
    click.echo(prettify(napi.get_user(model_id)))


@cli.command()
def account():
    """Get all information about your account!"""
    click.echo(prettify(napi.get_account()))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
def models(tournament):
    """Get map of account models!"""
    click.echo(prettify(napi.get_models(tournament)))


@cli.command()
@click.argument("username")
def profile(username):
    """Fetch the public profile of a user."""
    click.echo(prettify(napi.public_user_profile(username)))


@cli.command()
@click.argument("username")
def daily_model_performances(username):
    """Fetch daily performance of a model."""
    click.echo(prettify(napi.daily_model_performances(username)))


@cli.command()
def transactions():
    """List all your deposits and withdrawals."""
    click.echo(prettify(napi.wallet_transactions()))


@cli.command()
@click.option('--tournament', default=8,
              help='The ID of the tournament, defaults to 8')
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
@click.argument('path', type=click.Path(exists=True))
def submit(path, tournament, model_id):
    """Upload predictions from file."""
    click.echo(napi.upload_predictions(
        path, tournament, model_id))


@cli.command()
@click.argument("username")
def stake_get(username):
    """Get stake value of a user."""
    click.echo(napi.stake_get(username))


@cli.command()
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def stake_drain(model_id):
    """Completely remove your stake."""
    click.echo(napi.stake_drain(model_id))


@cli.command()
@click.argument("nmr")
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def stake_decrease(nmr, model_id):
    """Decrease your stake by `value` NMR."""
    click.echo(napi.stake_decrease(nmr, model_id))


@cli.command()
@click.argument("nmr")
@click.option(
    '--model_id', type=str, default=None,
    help="An account model UUID (required for accounts with multiple models")
def stake_increase(nmr, model_id):
    """Increase your stake by `value` NMR."""
    click.echo(napi.stake_increase(nmr, model_id))


@cli.command()
def version():
    """Installed numerapi version."""
    print(numerapi.__version__)


if __name__ == "__main__":
    cli()

*/




//
