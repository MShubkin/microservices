echo "Прогон not-fixed-channel-count-direct-reply теста..."

output_path="../output/not-fixed-channel-count-direct-reply.md"

cargo run --quiet --release --bin rabbit-listener --features="highload-tests" -- .env.highload > $output_path 2>&1 &
if [ $? -ne 0 ]; then
    echo "Ошибка при старте слушателя очереди, проверьте $output_path"
    exit 1
fi
echo "Слушатель начал свою работу"

cargo run --quiet --release --bin rabbit-not-fixed-channel-count-publisher --features="highload-tests" -- .env.highload >> $output_path 2>&1 &
if [ $? -ne 0 ]; then
    echo "Ошибка при старте паблишера, проверьте $output_path"
    exit 1
fi
echo "Паблишер начал свою работу"

wait

cat ../../../.env.highload >> $output_path

cat $output_path
