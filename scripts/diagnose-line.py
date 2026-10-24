"""Inspect Swarm/Mongo startup using whitelisted output, never raw env or logs."""
import json
import os
import re
import subprocess
from urllib.parse import parse_qs, urlsplit


def report(name, value):
    print(name + '=' + json.dumps(value, ensure_ascii=True), flush=True)


def docker(*args, timeout=20):
    return subprocess.run(['docker', *args], capture_output=True, text=True, timeout=timeout)


def inspect(*args):
    result = docker(*args)
    if result.returncode:
        return None
    return json.loads(result.stdout)[0]


def environment(values):
    return dict(value.split('=', 1) for value in values if '=' in value)


def main():
    stack = os.environ.get('STACK') or os.environ.get('APP_NAME') or 'alex-line'
    if not re.fullmatch(r'[a-zA-Z0-9][a-zA-Z0-9_-]*', stack):
        raise ValueError('Invalid stack name')
    backend = inspect('service', 'inspect', stack + '_backend')
    mongo = inspect('service', 'inspect', stack + '_mongo')
    if backend is None or mongo is None:
        report('error', 'Docker access required or service missing; run with sudo.')
        return 1
    task = backend['Spec']['TaskTemplate']
    spec = task['ContainerSpec']
    values = environment(spec.get('Env', []))
    report('backend_image', spec['Image'])
    report('backend_update_state', backend.get('UpdateStatus', {}).get('State', 'none'))
    report('backend_constraints', task.get('Placement', {}).get('Constraints', []))
    report('shared_overlay_network', bool(
        {network['Target'] for network in task.get('Networks', [])}
        & {network['Target'] for network in mongo['Spec']['TaskTemplate'].get('Networks', [])}))
    uri = urlsplit(values.get('MONGODB_URI', ''))
    report('backend_database_host', uri.hostname)
    report('backend_auth_source', parse_qs(uri.query).get('authSource', []))
    expected = 'mongodb://{}:{}@mongo:27017/?authSource=admin'.format(
        os.environ.get('MONGO_USER', ''), os.environ.get('MONGO_PASSWORD', ''))
    report('backend_database_uri_matches_env', values.get('MONGODB_URI') == expected)
    logs = docker('service', 'logs', '--tail', '40', stack + '_backend')
    report('backend_database_failure_logged', 'Database unavailable' in logs.stdout + logs.stderr)
    containers = docker('ps', '--filter', 'label=com.docker.swarm.service.name=' + stack + '_mongo', '--format', '{{.ID}}')
    ids = containers.stdout.splitlines()
    if not ids:
        report('mongo_running', False)
        return 1
    container = inspect('inspect', ids[0])
    if container is None:
        report('mongo_running', False)
        return 1
    report('mongo_running', container['State']['Running'])
    report('mongo_health', container['State'].get('Health', {}).get('Status', 'unknown'))
    mongo_env = environment(container['Config'].get('Env', []))
    report('mongo_credentials_match_env',
           mongo_env.get('MONGO_INITDB_ROOT_USERNAME') == os.environ.get('MONGO_USER')
           and mongo_env.get('MONGO_INITDB_ROOT_PASSWORD') == os.environ.get('MONGO_PASSWORD'))
    # Credentials stay inside the existing Mongo container. Probe results omit messages/URIs.
    script = r'''
for (const [test, host] of [['auth_local','127.0.0.1'],['service_dns','mongo'],['tasks_dns','tasks.mongo']]) {
  try {
    const conn = new Mongo('mongodb://' + host + ':27017/?serverSelectionTimeoutMS=4000&connectTimeoutMS=4000&directConnection=true');
    const admin = conn.getDB('admin');
    const result = admin.auth({user: process.env.MONGO_INITDB_ROOT_USERNAME, pwd: process.env.MONGO_INITDB_ROOT_PASSWORD});
    const ok = (result === 1 || result.ok === 1) && admin.runCommand({ping:1}).ok === 1;
    print('mongo_probe=' + JSON.stringify({test, ok}));
  } catch (error) {
    print('mongo_probe=' + JSON.stringify({test, ok:false, code:error.code || null, kind:error.name || 'unknown'}));
  }
}
'''
    probe = docker('exec', ids[0], 'mongosh', '--nodb', '--quiet', '--eval', script, timeout=40)
    for line in probe.stdout.splitlines():
        if line.startswith('mongo_probe='):
            value = json.loads(line.split('=', 1)[1])
            safe = {key: value[key] for key in ['test', 'ok', 'code', 'kind'] if key in value}
            report('mongo_probe', safe)
    report('probe_exit_code', probe.returncode)
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired):
        report('error', 'Diagnostic command failed or timed out; no service configuration was changed.')
        raise SystemExit(1)
