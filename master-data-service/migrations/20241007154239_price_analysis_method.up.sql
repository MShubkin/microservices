-- Table: public.price_analysis_method
CREATE TABLE public.price_analysis_method
(
	id SMALLSERIAL PRIMARY KEY,
	uuid uuid NOT NULL,
	name VARCHAR(250) NOT NULL DEFAULT '',
    is_removed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    changed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    created_by INTEGER NOT NULL,
    changed_by INTEGER NOT NULL
);

COMMENT ON TABLE public.price_analysis_method IS 'Справочник "Метод ценообразования"';

COMMENT ON COLUMN public.price_analysis_method.uuid IS 'UID Записи';
COMMENT ON COLUMN public.price_analysis_method.id IS 'ID Записи';
COMMENT ON COLUMN public.price_analysis_method.name IS 'Наименование';
COMMENT ON COLUMN public.price_analysis_method.is_removed IS 'Запись удалена';

COMMENT ON COLUMN public.price_analysis_method.changed_at IS 'Дата и время изменения. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.price_analysis_method.created_at IS 'Дата и время создания. Автоматически присваивается при создании объекта';
COMMENT ON COLUMN public.price_analysis_method.changed_by IS 'Код пользователя ответственного исполнителя, изменившего объект';
COMMENT ON COLUMN public.price_analysis_method.created_by IS 'Код пользователя ответственного исполнителя, создавшего объект';

INSERT INTO public.price_analysis_method(uuid, name, changed_at, created_at, changed_by, created_by)
	VALUES
	('c334da23-0daa-4163-86b0-bc1d68f03bb7', 'Метод сопоставимых рыночных цен (анализ рынка)', now(), now(), 1, 1),
	('2edc692b-cbaa-42d0-9154-feccbbb37472', 'Метод удельных показателей (параметрический)', now(), now(), 1, 1),
	('18612bc2-533e-4f18-ba7b-890de1f4f25b', 'Затратный метод', now(), now(), 1, 1),
	('9ab1455d-308c-4544-8dfc-96fc36912a54', 'Тарифный метод', now(), now(), 1, 1),
	('0103a681-9b75-4623-a31d-b4780482068c', 'Проектно-сметный метод', now(), now(), 1, 1),
	('b4fdf990-5a38-4d17-9f88-8897d3dc673f', 'Метод расчета цены НИОКР', now(), now(), 1, 1),
	('977b5841-de13-4a70-a0da-800774c877af', 'Метод формирования цены с учетом внешних факторов', now(), now(), 1, 1),
	('0bbc75e8-7dad-4bdc-a668-42f1ea80f4be', 'Метод формирования цены на товары для машиностроительной отрасли длительного производства', now(), now(), 1, 1)
;
