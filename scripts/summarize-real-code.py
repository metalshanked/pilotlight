"""Summarize fixed replays without turning timeouts or occurrence counts into accuracy."""
import argparse
import collections
import csv
import hashlib
import json
from pathlib import Path
from urllib.parse import quote


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def csv_write(path, rows):
    with path.open('w', newline='', encoding='utf-8') as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--before', default='pilotlight-before')
    parser.add_argument('--after', default='pilotlight-release')
    parser.add_argument('--cisco', default='cisco-main-parallel')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    runs = {name: {r['index']: r for r in read(args.work / directory / 'outcomes.json')}
            for name, directory in [('before', args.before), ('after', args.after), ('cisco', args.cisco)]}
    expected = set(range(1, len(read(args.work / 'corpus.json')) + 1))
    assert all(set(rows) == expected for rows in runs.values()), 'Incomplete replay: retain pending outcomes'
    rows, added_rows, removed_rows = [], [], []
    added_types, removed_types = collections.Counter(), collections.Counter()
    added_test_types = collections.Counter()
    policy_path = args.work / "cisco-test-paths.json"
    policy = read(policy_path) if policy_path.exists() else None
    test_paths = {k: set(v) for k, v in policy["repositories"].items()} if policy else {}
    for index in sorted(expected):
        old, new, cisco = (runs[key][index] for key in ['before', 'after', 'cisco'])
        assert old['commit'] == new['commit'] == cisco['commit']
        assert old['source_inventory_sha256'] == new['source_inventory_sha256'] == cisco['source_inventory_sha256']
        assert old['status'] in ['partial', 'complete_within_declared_scope']
        assert new['status'] in ['partial', 'complete_within_declared_scope']
        complete = cisco['status'] in ['completed', 'completed_with_warnings']
        row = {'index': index, 'group': new['group'], 'repository': new['repository'], 'commit': new['commit'],
               'source_inventory_sha256': new['source_inventory_sha256'], 'source_omitted_links': new['source_omitted_links'], 'pilotlight_before_status': old['status'],
               'pilotlight_after_status': new['status'], 'cisco_status': cisco['status'],
               'pilotlight_before_occurrences': old['findings'], 'pilotlight_after_occurrences': new['findings'],
               'cisco_assembled_components': cisco.get('findings') if complete else None,
               'pilotlight_after_replay_seconds': new['process_wall_seconds'],
               'cisco_replay_seconds': cisco['process_wall_seconds'],
               'pilotlight_files_scanned': new['coverage']['files_scanned'],
               'pilotlight_files_failed': new['coverage']['files_failed'],
               'pilotlight_diagnostics': len(new['coverage']['diagnostics']),
               'cisco_warnings': cisco.get('warnings') if complete else None}
        for kind in ['agent', 'mcp_client', 'mcp_server', 'mcp_gateway', 'tool', 'model', 'dependency', 'skill']:
            for key, record in [('before', old), ('after', new), ('cisco', cisco)]:
                row[f'{key}_{kind}'] = record.get('by_type', {}).get(kind, 0) if key != 'cisco' or complete else None
        rows.append(row)
        prior = {a['id']: a for a in read(args.work / args.before / f'{index:03}.json')['assets']}
        current = {a['id']: a for a in read(args.work / args.after / f'{index:03}.json')['assets']}
        cisco_files = set()
        root = str((args.work / 'sources' / f'{index:03}' / 'source').resolve()).replace('\\', '/') + '/'
        if complete:
            for component in read(args.work / args.cisco / f'{index:03}.json')['components']:
                metadata = component.get('metadata') or {}
                paths = [component['file_path']] + (metadata.get('evidence_files') or [])
                paths += [item.get('file', '') for item in (metadata.get('evidence') or []) if isinstance(item, dict)]
                for path in paths:
                    if not isinstance(path, str):
                        continue
                    normalized = path.replace('\\', '/')
                    if normalized.casefold().startswith(root.casefold()):
                        normalized = normalized[len(root):]
                    cisco_files.add((component['component_type'], normalized))
        for ids, assets, target, totals in [(current.keys() - prior.keys(), current, added_rows, added_types),
                                          (prior.keys() - current.keys(), prior, removed_rows, removed_types)]:
            for identity in sorted(ids):
                asset = assets[identity]
                totals[asset['kind']] += 1
                in_tests = all(e['path'] in test_paths[f'{index:03}'] for e in asset['evidence']) if policy else None
                if target is added_rows and in_tests:
                    added_test_types[asset['kind']] += 1
                for evidence in asset['evidence']:
                    path = evidence['path']
                    target.append({'repository': new['repository'], 'commit': new['commit'], 'asset_id': identity,
                                   'kind': asset['kind'], 'framework': asset.get('framework'), 'name': asset['name'],
                                   'path': path, 'line': evidence['line'], 'column': evidence.get('column'),
                                   'mechanism': evidence['mechanism'],
                                   'source_url': f"{new['repository']}/blob/{new['commit']}/{quote(path, safe='/')}#L{evidence['line']}",
                                   'packs': ';'.join(sorted({r['pack']['id'] for r in asset.get('detection_rules', [])})),
                                   'cisco_status': cisco['status'], 'in_cisco_test_paths': in_tests,
                                   'cisco_same_type_in_file': (asset['kind'], path) in cisco_files if complete else None,
                                   'comparison_limit': 'file/type presence only; test-scope and assembly differ; not a scored miss'})
    meta = {name: read(args.work / directory / 'run.json') for name, directory in
            [('before', args.before), ('after', args.after), ('cisco', args.cisco)]}
    if policy:
        assert policy['cisco_commit'] == meta['cisco']['cisco_commit']
    summary = {'added_occurrences_in_test_paths_by_type': dict(added_test_types) if policy else None,
               'test_path_policy_sha256': hashlib.sha256(policy_path.read_bytes()).hexdigest() if policy else None,
               'repositories': len(rows), 'groups': dict(collections.Counter(r['group'] for r in rows)),
               'all_source_inventories_identical': True,
               'pilotlight_before_outcomes': dict(collections.Counter(r['status'] for r in runs['before'].values())),
               'pilotlight_after_outcomes': dict(collections.Counter(r['status'] for r in runs['after'].values())),
               'cisco_outcomes': dict(collections.Counter(r['status'] for r in runs['cisco'].values())),
               'added_occurrences_by_type': dict(added_types), 'removed_occurrences_by_type': dict(removed_types),
               'rows': rows, 'run_metadata': meta,
               'qualification': 'Unlabeled development corpus, not held out. Counts are occurrences versus assembled components. Cisco test-only filtering differs. Timeouts are null counts, never zero. Timing includes source checks on a shared workstation, not controlled speed or accuracy.'}
    (args.output / 'real-code-comparison.json').write_text(json.dumps(summary, indent=2), encoding='utf-8')
    csv_write(args.output / 'real-code-comparison.csv', rows)
    if added_rows:
        csv_write(args.output / 'real-code-new-evidence.csv', added_rows)
    (args.output / 'real-code-removed-evidence.json').write_text(json.dumps(removed_rows, indent=2), encoding='utf-8')
    print(json.dumps({k: v for k, v in summary.items() if k not in ['rows', 'run_metadata']}, indent=2))


if __name__ == '__main__':
    main()
