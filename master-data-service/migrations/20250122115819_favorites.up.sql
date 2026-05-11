CREATE TABLE public.favorite_list (
    uuid UUID PRIMARY KEY,
    user_id INT4 NOT NULL,
    dictionary_id INT4 NOT NULL,
    dictionary_item_id INT4 NOT NULL
);

CREATE TABLE public.favorite_dictionary (
    uuid Uuid PRIMARY KEY,
    id INT4 NOT NULL,
    text VARCHAR NOT NULL DEFAULT '',
    name VARCHAR NOT NULL DEFAULT ''
);

INSERT INTO public.favorite_dictionary (uuid, id, text, name) VALUES
('fb98e992-f1d7-4bbb-820d-322dbaab8aca', 1, 'Избранные для Справочника Экспертов АЦ', 'PricingExpert'),
('8ca2462b-24ef-47f2-96d2-2f7d3099de1f', 2, 'Избранные для Справочника Экспертов ПД', 'SpecializedDepartmentsExpert');

COMMENT ON TABLE public.favorite_list IS 'Избранные записи';
COMMENT ON COLUMN public.favorite_list.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.favorite_list.user_id IS 'Идентификатор пользователя, которому принадлежит избранная запись';
COMMENT ON COLUMN public.favorite_list.dictionary_id IS 'Идентификатор справочника, для которого создаются Избранные записи';
COMMENT ON COLUMN public.favorite_list.dictionary_item_id IS 'Идентификатор записи справочника';

COMMENT ON TABLE public.favorite_dictionary IS 'Избранные справочники';
COMMENT ON COLUMN public.favorite_dictionary.uuid IS 'Уникальный идентификатор записи';
COMMENT ON COLUMN public.favorite_dictionary.id IS 'Идентификатор справочника, для которого создаются Избранные записи';
COMMENT ON COLUMN public.favorite_dictionary.text IS 'Текстовое наименование (описание) записи Избранного справочника';
COMMENT ON COLUMN public.favorite_dictionary.name IS 'Наименование справочника';
