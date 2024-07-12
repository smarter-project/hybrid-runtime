# Running container using hybrid runtime + k3s

For this demo, we'll be using the [AVH setup](./avh_setup.md)

## Download the hybrid-runtime
Once your AVH model is setup, we already have a pre-built hybrid-runtime and some necessary scripts packaged in a GitHub package, to download this package, run:
```
wget https://github.com/smarter-project/hybrid-runtime/releases/download/v1.5/hybrid.tar.gz 
```
Extract the files to /usr/local/bin using: 
```
tar -C /usr/local/bin/ -xvf hybrid.tar.gz
```


## K3s Setup

We’ll be using a single node k3s cluster setup, to download k3s and set it up: (this command could take few minutes to run)
```
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="server --disable traefik --disable metrics-server --disable coredns --disable local-storage --flannel-backend=none --cluster-dns 169.254.0.2 \
--container-runtime-endpoint=unix://var/run/containerd/containerd.sock" sh -s -
```
Make sure k3s is running:
```
systemctl status k3s
```

Next, we need to make k3s aware of the hybrid-runtime, to do so we need to update containerd config file, we’ve packaged the config file with the k3s example YAML files in GitHub 

Download the k3s demo example YAML files: 
```
wget https://github.com/smarter-project/hybrid-runtime/releases/download/v1.5/example.tar.gz 
```
Extract the files: (the tar file will be extracted to an example folder)
```
tar -xvf example.tar.gz
```

Create a containerd directory under /etc and copy the config file there:
```
mkdir /etc/containerd
mv example/config.toml /etc/containerd/ 
```
You need to restart containerd:
```
systemctl restart cotnainerd
```
Make sure containerd is running: 
```
systemctl status containerd
```
Now if you run:
```
kubectl get nodes 
```

You will see that the node is not ready 
```
NAME     STATUS     ROLES                  AGE   VERSION
narsil   NotReady   control-plane,master   18m   v1.29.6+k3s2
```
To fix this, you need to apply a CNI, we’ll be using the smarter CNI, and label the node. Run:
```
kubectl apply -f example/smarter_cni.yaml
kubectl label node narsil smarter.cni=deploy
```

Rerun kubectl get nodes, this time you should be able to see that the node is ready:
```
root@narsil:~# kubectl get nodes 
NAME     STATUS   ROLES                  AGE   VERSION
narsil   Ready    control-plane,master   24m   v1.29.6+k3s2
```

## K3s Demo

### Deploy smarter camera demo

For the k3s demo, we’ll be using the smarter camera demo.

First, we need to set a runtimeClass in k3s, it allows us to select the container runtime we want to use.
```
kubectl apply -f example/runtime_class.yaml
```
Once this is done, we can run the smarter demo: 
```
kubectl apply -f example/test_hybrid.yaml
```
The `test_hybrid.yaml` file contains the following: 
```
kind: Pod
apiVersion: v1
metadata:
  name: example3
  labels:
    k3s-app: example3
spec:
    runtimeClassName: hybrid          å
    containers:
      - name: example-hybrid-pod3
        image: ghcr.io/smarter-project/smart-camera-hybrid-application/hybrid_app_imx8mp:latest
        imagePullPolicy: IfNotPresent
```
å
You can check that the firmware is running either by:
- Go to the Cortex-M Console and you should see a timestamp output.
-	Run kubectl get pods -A 
```
root@narsil:~# kubectl get pods -A 
NAMESPACE     NAME                READY   STATUS    RESTARTS       AGE
default       example3            1/1     Running   0              6m57s
kube-system   smarter-cni-wplzn   1/1     Running   3 (141m ago)   4h29m 
```
A pod with the name example3 should be running.
### Kill the demo
To kill the demo, run: 
```
kubectl delete pod example3 --grace-period=0 --force
```
Make sure the pod was terminated: (the termination process takes few minutes)
-	Go to the Cortex-M Console, same as before and check that there are no new outputs.
-	Check that the firmware is offline: 
```
root@narsil:~# cat /sys/class/remoteproc/remoteproc0/state 
offline
```
-	Make sure the created pod above was deleted:
```
root@narsil:~# kubectl get pods -A 
NAMESPACE     NAME                READY   STATUS    RESTARTS       AGE
kube-system   smarter-cni-wplzn   1/1     Running   3 (143m ago)   4h31m
```
-	Make sure all the container resources were deleted:
```
ls /var/lib/hybrid-runtime/
```