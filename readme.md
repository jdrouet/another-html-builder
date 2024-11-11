# Another HTML builder

The goal of this builder is to be simple, to only rely on the standard library, to avoid copying values and write directly to a buffer.
There is no lock involved, the ownership of the buffer is the only thing that will avoid race conditions.
