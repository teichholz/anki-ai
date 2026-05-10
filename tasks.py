from invoke import task


@task
def check(ctx):
    """Run ty, ruff lint, and ruff format."""
    ctx.run("ty check .", pty=True)
    ctx.run("ruff check .", pty=True)
    ctx.run("ruff format .", pty=True)
