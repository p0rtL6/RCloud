for i in {1..10}; do
    mkdir files/dir$i
    for e in {1..10}; do
        touch files/dir$i/file$e
    done
done

