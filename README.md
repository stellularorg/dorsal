# 🐬 dorsal

Dorsal is the backbone server structure of [Stellular projects](https://code.stellular.org/stellular)! It contains the base SQL helper modules and basic server helpers that make projects supporting `sqlite`, `mysql`, and `postgresql` easy to start and expand.

* Server crate located in `src/`
* Example/template project located in `dorsal_example/`

The Dorsal example also includes support for the user authentication base that is provided by [Guppy](https://code.stellular.org/stellular/guppy), however this is not required.

## Usage

Add the package as a dependency:

```
cargo add dorsal
```

You can then model after the template project.

## Template Only

You can clone only the `dorsal_example/` directory with the command below:

```bash
PROJECT_NAME="dorsal_example" &&
git clone --depth=1 https://github.com/stellularorg/dorsal $PROJECT_NAME &&
cd $PROJECT_NAME &&
git sparse-checkout set --no-cone dorsal_example &&
mv dorsal_example ../tmpdex &&
cd ../ &&
sudo rm -r $PROJECT_NAME &&
mv ./tmpdex $PROJECT_NAME &&
cd $PROJECT_NAME
```
