

CREATE TABLE public.number_range(
  object_type SMALLINT NOT NULL PRIMARY KEY,
  start_idx BIGINT NOT NULL,
  end_idx BIGINT NOT NULL,
  next_idx BIGINT NOT NULL
) TABLESPACE pg_default;

INSERT INTO public.number_range VALUES 
  (0, 0, 8799999999, 0),
  (2, 8900000000, 8999999999, 8900000000),
  (1, 8800000000, 8899999999, 8800000000);

COMMENT ON TABLE public.number_range is 'Таблица доступных чисел для порядковых номеров';

COMMENT ON COLUMN public.number_range.object_type is 'Значения: 
Повестка
Протокол';

COMMENT ON COLUMN public.number_range.start_idx is 'Значения: 
8800000000 - Повестка 
8900000000 - Протокол очного заседания СК / Протокол заочного заседания СК';

COMMENT ON COLUMN public.number_range.end_idx is 'Значения: 
8899999999 - Повестка 
8999999999 - Протокол очного заседания СК / Протокол заочного заседания СК';
