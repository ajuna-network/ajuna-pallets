#!/usr/bin/python3

import requests
import json
import time
import csv

event_url = 'https://bajun.api.subscan.io/api/scan/event'

if __name__ == "__main__":
    # We read the event ids and generate account chains from it

    event_id_list = []
    event_accounts_list = []

    affiliatee_chains = {}

    with open('event-ids.csv', 'r') as csvfile:
        reader = csv.reader(csvfile, dialect='unix', lineterminator=',\n')

        for row in reader:
            event_id_list.append(row[0])

    headers = {
        'User-Agent': 'Apidog/1.0.0 (https://apidog.com)',
        'Content-Type': 'application/json'
    }

    for event_id in event_id_list:
        print('Processing event ', event_id)

        payload = json.dumps({
            'event_index': event_id
        })

        response = requests.request('POST', event_url, headers=headers, data=payload)
        time.sleep(0.1)

        response_data = response.json()

        print(response_data['message'])

        event_params = response_data['data']['params']

        affiliator = ""
        affiliatee = ""

        if event_params[0]['name'] == 'to':
            affiliator = event_params[0]['value']
            affiliatee = event_params[1]['value']
        else:
            affiliator = event_params[1]['value']
            affiliatee = event_params[0]['value']
        
        affiliatee_chains[affiliator] = []
        affiliatee_chains[affiliatee] = []

        event_accounts_list.append({
            'affiliator': affiliator,
            'affiliatee': affiliatee,
        })
    
    print('Writing rows to file...')

    with open('event-accounts.csv', 'w') as csvfile:
        writer = csv.DictWriter(csvfile, dialect='unix', fieldnames=['affiliator', 'affiliatee'])

        writer.writeheader()

        for event_accounts in event_accounts_list:
            writer.writerow(event_accounts)
