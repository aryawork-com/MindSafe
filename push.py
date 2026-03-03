import os
import re
import subprocess
import datetime
from pathlib import Path
from dotenv import load_dotenv
from tabulate import tabulate


# Load environment variables
load_dotenv()

APP_NAME = "MindSafe"

VERSION_FILE = Path("src/version.rs")


def get_current_datetime():
    now = datetime.datetime.now()
    return now.strftime("%Y-%m-%d %H:%M:%S %Z"), now.strftime("%y%m%d%H%M")


def get_commit_message():
    message = input("Enter commit message: ").strip()
    return message if message else "Blind Commit"


def stage_changes():
    subprocess.run(["git", "add", "."], check=True)


def count_staged_changes():
    result = subprocess.run(
        ["git", "diff", "--cached", "--numstat"],
        stdout=subprocess.PIPE,
        text=True
    )
    return len(result.stdout.strip().splitlines())


def categorize_change(num_changes):
    if num_changes <= 30:
        return "lite"
    elif num_changes <= 100:
        return "small"
    else:
        return "large"


def read_current_version():
    content = VERSION_FILE.read_text()
    match = re.search(r'pub const VERSION: &str = "(.*?)";', content)
    if not match:
        raise ValueError("VERSION line not found in version.rs")
    return match.group(1), content


def bump_version(current_version, mark, timestamp):
    base_version = current_version.split(":")[0]
    major, minor, patch = map(int, base_version.split("."))

    if mark == "lite":
        patch += 1
    elif mark == "small":
        minor += 1
    else:
        major += 1

    return f"{major}.{minor}.{patch}:{timestamp}"


def update_version_file(content, new_version):
    new_line = f'pub const VERSION: &str = "{new_version}";'
    new_content = re.sub(
        r'^pub const VERSION: &str = ".*?";',
        new_line,
        content,
        flags=re.MULTILINE
    )
    VERSION_FILE.write_text(new_content)


def commit_and_push(commit_message):
    subprocess.run(["git", "add", "."], check=True)
    subprocess.run(["git", "commit", "-m", commit_message], check=True)
    subprocess.run(["git", "push"], check=True)


def main():
    print("\n---------Initiating---------\n")

    current_datetime, timestamp = get_current_datetime()
    commit_message = get_commit_message()

    stage_changes()
    num_changes = count_staged_changes()
    # print(f"Total Number of Changes: {num_changes}")

    mark = categorize_change(num_changes)
    # print(f"The changes are categorized as {mark}")

    current_version, version_content = read_current_version()
    new_version = bump_version(current_version, mark, timestamp)
    update_version_file(version_content, new_version)

    # print(f"Version updated from {current_version} to {new_version}")

    formatted_message = (
        f"{current_datetime}-({mark})-({num_changes})-(V:{new_version})-> \"{commit_message}\""
    )

    commit_and_push(formatted_message)


    print("\n---------Summary---------\n")
    print(tabulate([['Date', current_datetime], ['App Name', APP_NAME], ['Number of Changes', num_changes], ['Changes Category', mark.capitalize()], ['Old Version', current_version], ['New Version', new_version], ['Commit Message', commit_message.capitalize()]],
                   headers=['Parameter', 'Value']))
    print("\n---------END---------\n")


if __name__ == "__main__":
    main()
