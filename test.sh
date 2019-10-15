## Test Config
#domain: 127.0.0.1
#port: 8000
#ssl_private_key: null
#ssl_cert_chain: null
#workers: 10
#secret: '72558847d57c22a2f19d711537cdc446'
#basic_auth_user: "test"
#basic_auth_password: "testtest"
#basic_auth_and_secret: false
#webhooks:
#  -
#    name: 'ls'
#    command: '/bin/ls {{param1}} {{param2}}'
#    cwd: '/home/nuke'
#    mode: ''
#    parallel_processes: 2
#  -
#    name: 'lshome'
#    command: '/bin/ls /home'
#    cwd: '/home/nuke'
#    mode: ''
#    parallel_processes: 2

echo -n '{"parameters":{"param1":"-al","param2":"/tmp"}}' | http POST localhost:8000/ls \
    X-Hub-Signature:'sha1=d762407ca7fb309dfbeb73c080caf6394751f0a4' \
    Authorization:'Basic dGVzdDp0ZXN0dGVzdA=='

http GET localhost:8000/lshome \
    Authorization:'Basic dGVzdDp0ZXN0dGVzdA=='
