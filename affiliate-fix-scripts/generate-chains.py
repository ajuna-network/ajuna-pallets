#!/usr/bin/python3

import csv

affiliate_max_level = 2

if __name__ == "__main__":
    event_accounts_list = []

    affiliatee_chains = {}

    with open('event-accounts.csv', 'r') as csvfile:
        reader = csv.DictReader(csvfile)

        for row in reader:
            event_accounts_list.append(row)

    for event_accounts in event_accounts_list:
        affiliator = event_accounts['affiliator']
        affiliatee = event_accounts['affiliatee']

        chain = affiliatee_chains.get(affiliator, []).copy()

        if len(chain) == affiliate_max_level:
            chain.pop()

        affiliatee_chains[affiliatee] = [affiliator] + chain

    print('Writing rows to file...')

    with open('affiliatee-chains.csv', 'w') as csvfile:
        writer = csv.writer(csvfile, dialect='unix', lineterminator=',\n')
        for account_id in affiliatee_chains:
            writer.writerow([account_id] + affiliatee_chains[account_id])



