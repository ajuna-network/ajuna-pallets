#!/usr/bin/python3

import requests
import json
import time
import csv

event_list_url = 'https://bajun.api.subscan.io/api/v2/scan/events'

page_idx = 0
event = 'AccountAffiliated'

row_size = 100

event_url = 'https://bajun.api.subscan.io/api/scan/event'

if __name__ == '__main__':
    # First we collect all relevant event id
    event_id_list = []

    payload = json.dumps({
        'page': page_idx,
        'row': row_size,
        'event_id': event
    })
    headers = {
        'User-Agent': 'Apidog/1.0.0 (https://apidog.com)',
        'Content-Type': 'application/json'
    }

    response = requests.request('POST', event_list_url, headers=headers, data=payload)
    page = response.json()
    time.sleep(0.2)

    total_entries = int(page['data']['count'])

    print('Total entries: ', total_entries)


    for entry in page['data']['events']:
        event_id_list.append(entry['event_index'])

    visited_entries = len(page['data']['events'])
    
    print('Visited entries: ', visited_entries)

    while visited_entries < total_entries:
        page_idx += 1

        payload = json.dumps({
            'page': page_idx,
            'row': row_size,
            'event_id': event
        })

        response = requests.request('POST', event_list_url, headers=headers, data=payload)
        page = response.json()
        time.sleep(0.5)

        print(page['message'])

        for entry in page['data']['events']:
            event_id_list.append(entry['event_index'])
        
        visited_entries += len(page['data']['events'])

        print('Visited entries: ', visited_entries)

    event_id_list.reverse()

    print('Writing rows to file...')

    with open('event-ids.csv', 'w') as csvfile:
        writer = csv.writer(csvfile, dialect='unix', lineterminator=',\n')
        for event_id in event_id_list:
            writer.writerow([event_id])
